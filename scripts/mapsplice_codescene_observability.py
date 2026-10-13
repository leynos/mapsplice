"""Emit bounded, payload-free observations for one CodeScene policy audit.

For example, an ``AuditSession`` records a request's elapsed time and status
without retaining its endpoint, headers, response body, or exception message.
"""

from __future__ import annotations

import json
import math
import sys
import time
from collections.abc import Callable, Iterator
from contextlib import contextmanager, nullcontext
from typing import TypeVar

from mapsplice_codescene_models import MAX_PAGES, AuditOperation, VerificationError

T = TypeVar("T")
OPERATIONS = frozenset(
    {
        "environment",
        "repository",
        "policy_page",
        "policy_readback",
        "environment_readback",
    }
)
CODES = frozenset(
    {
        "repository",
        "repository-identity",
        "api-unavailable",
        "api-response",
        "access-denied",
        "not-found",
        "api-status",
        "api-shape",
        "environment-shape",
        "environment-policy",
        "environment-drift",
        "policy-shape",
        "policy-cap",
        "policy-drift",
        "policy-count",
        "policy-page",
        "policy-identity",
        "pagination-link",
        "pagination-cycle",
    }
)
OUTCOMES = (
    "success",
    "failure",
    "timeout",
    "access-denied",
    "policy-drift",
    "environment-drift",
    "policy-cap",
)


def _stderr_event(event: dict[str, object]) -> None:
    """Emit trusted JSON to stderr; ``_stderr_event({})`` prints ``{}``."""
    print(json.dumps(event, sort_keys=True), file=sys.stderr)


def _code(error: BaseException | None) -> str | None:
    """Bound error labels; ``_code(None)`` returns ``None``."""
    if error is None:
        return None
    if (
        isinstance(error, VerificationError)
        and isinstance(error.code, str)
        and error.code in CODES
    ):
        return error.code
    return "unexpected"


def _outcome(error: BaseException | None) -> str:
    """Classify safe outcomes; ``_outcome(None)`` returns ``"success"``."""
    if error is None:
        return "success"
    if isinstance(error, VerificationError) and error.failure_kind == "timeout":
        return "timeout"
    code = _code(error)
    return code if code in OUTCOMES else "failure"


def _status(response: object) -> int | None:
    """Accept HTTP integers only; ``_status(None)`` returns ``None``."""
    value = getattr(response, "status", None)
    return value if type(value) is int and 100 <= value <= 599 else None


def _bounded_count(value: int) -> int:
    """Cap audit counts; ``_bounded_count(1000)`` returns ``110``."""
    return min(value, MAX_PAGES + 10)


class AuditSession:
    """Collect fixed-vocabulary observations for one bounded verification run.

    ``clock`` and ``emit`` may be injected for deterministic offline tests;
    default events are JSON lines on stderr alongside the existing diagnostics.
    """

    def __init__(
        self,
        *,
        clock: Callable[[], float] = time.monotonic,
        emit: Callable[[dict[str, object]], None] = _stderr_event,
    ) -> None:
        """Start fresh bounded counters with an injectable clock and event sink.

        Parameters
        ----------
        clock : Callable[[], float], optional
            Monotonic clock returning seconds, defaulting to ``time.monotonic``.
        emit : Callable[[dict[str, object]], None], optional
            Sink receiving sanitized event dictionaries; the default writes JSON
            lines on stderr.

        Examples
        --------
        ``AuditSession(emit=events.append)`` starts with zero requests and sends
        observations to the supplied list instead of stderr.
        """
        self.clock = clock
        self.emit = emit
        self.started = clock()
        self.requests = 0
        self.page_reads = 0
        self.pages: set[int] = set()
        self.request_latency_ms = 0.0
        self.request_outcomes = dict.fromkeys(OUTCOMES, 0)
        self.stage_outcomes = dict.fromkeys(OUTCOMES, 0)

    def _elapsed(self, started: float) -> float:
        """Clamp timing; ``self._elapsed(future)`` returns ``0.0`` for a future start."""
        elapsed = self.clock() - started
        if not math.isfinite(elapsed):
            return 0.0
        return round(min(86400.0, max(0.0, elapsed)) * 1000, 3)

    def _context(
        self, operation: AuditOperation, page: int | None
    ) -> dict[str, object]:
        """Bound labels; ``self._context("repository", None)`` keeps page ``None``."""
        safe_operation = operation if operation in OPERATIONS else "unknown"
        safe_page = page if type(page) is int and 1 <= page <= MAX_PAGES + 1 else None
        resource = {
            "policy_page": "deployment_policies",
            "policy_readback": "deployment_policies",
            "repository": "repository",
            "environment": "environment",
            "environment_readback": "environment",
        }.get(safe_operation, "unknown")
        return {"operation": safe_operation, "resource": resource, "page": safe_page}

    def request(
        self,
        fetch: Callable[[str], T],
        endpoint: str,
        operation: AuditOperation,
        page: int | None = None,
    ) -> T:
        """Observe a transport request using caller-supplied trusted context.

        ``session.request(fetch, endpoint, "environment")`` returns the fetched
        response unchanged and emits its bounded status and elapsed milliseconds.
        """
        started = self.clock()
        status = None
        error = None
        try:
            response = fetch(endpoint)
            status = _status(response)
            if status != 200:
                code = {403: "access-denied", 404: "not-found"}.get(
                    status, "api-response" if status is None else "api-status"
                )
                error = VerificationError(code, "request unsuccessful")
            return response
        except BaseException as failure:
            error = failure
            raise
        finally:
            elapsed = self._elapsed(started)
            context = self._context(operation, page)
            self.requests = _bounded_count(self.requests + 1)
            self.request_latency_ms = min(86400000.0, self.request_latency_ms + elapsed)
            if context["page"] is not None:
                self.page_reads = _bounded_count(self.page_reads + 1)
                self.pages.add(context["page"])
            outcome = _outcome(error)
            self.request_outcomes[outcome] = _bounded_count(
                self.request_outcomes[outcome] + 1
            )
            self.emit(
                {
                    "event": "codescene_request",
                    **context,
                    "status": status,
                    "error_code": _code(error),
                    "outcome": outcome,
                    "elapsed_ms": elapsed,
                }
            )

    @contextmanager
    def stage(
        self, operation: AuditOperation, page: int | None = None
    ) -> Iterator[None]:
        """Observe semantic validation, including successful-request readback drift.

        ``with session.stage("policy_readback", 1): validate()`` emits a stage
        outcome; a raised policy-drift error is recorded and propagated unchanged.
        """
        started = self.clock()
        error = None
        try:
            yield
        except BaseException as failure:
            error = failure
            raise
        finally:
            outcome = _outcome(error)
            self.stage_outcomes[outcome] = _bounded_count(
                self.stage_outcomes[outcome] + 1
            )
            self.emit(
                {
                    "event": "codescene_stage",
                    **self._context(operation, page),
                    "error_code": _code(error),
                    "outcome": outcome,
                    "elapsed_ms": self._elapsed(started),
                }
            )

    @contextmanager
    def run(self) -> Iterator[None]:
        """Emit a verdict even when verification exits at its first request.

        ``with session.run(): verify()`` emits one final summary on success or
        failure, including every request observed before an early rejection.
        """
        error = None
        try:
            yield
        except BaseException as failure:
            error = failure
            raise
        finally:
            self.emit(
                {
                    "event": "codescene_summary",
                    "outcome": _outcome(error),
                    "error_code": _code(error),
                    "elapsed_ms": self._elapsed(self.started),
                    "request_count": self.requests,
                    "page_count": len(self.pages),
                    "page_read_count": self.page_reads,
                    "request_latency_ms": round(self.request_latency_ms, 3),
                    "request_outcomes": self.request_outcomes,
                    "stage_outcomes": self.stage_outcomes,
                }
            )


def audit_stage(
    audit: AuditSession | None,
    operation: AuditOperation,
    page: int | None = None,
):
    """Keep legacy callers silent when no observation session was supplied."""
    return audit.stage(operation, page) if audit is not None else nullcontext()
