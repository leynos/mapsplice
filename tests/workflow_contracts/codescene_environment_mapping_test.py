"""Map GitHub environment responses to bounded typed protection values."""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))

from mapsplice_codescene_api import ApiResponse, environment
from mapsplice_codescene_models import EnvironmentProtection, VerificationError


@pytest.mark.parametrize(
    ("body", "expected"),
    [
        (
            {
                "name": "codescene",
                "deployment_branch_policy": {
                    "protected_branches": False,
                    "custom_branch_policies": True,
                },
            },
            EnvironmentProtection("codescene", False, True),
        ),
        (
            {
                "name": "codescene",
                "deployment_branch_policy": {
                    "protected_branches": 0,
                    "custom_branch_policies": True,
                },
            },
            EnvironmentProtection("codescene", None, True),
        ),
    ],
    ids=["valid-protection", "invalid-flag-is-unknown"],
)
def test_environment_response_maps_to_typed_protection(
    body: object, expected: EnvironmentProtection
) -> None:
    """Environment JSON fields are decoded without accepting coerced booleans."""
    assert environment(ApiResponse(200, {}, body)) == expected


@pytest.mark.parametrize(
    ("body", "code"),
    [
        (None, "api-shape"),
        ({"name": "codescene"}, "environment-shape"),
        ({"name": "codescene", "deployment_branch_policy": None}, "environment-shape"),
        ({"name": 42, "deployment_branch_policy": {}}, "environment-shape"),
    ],
    ids=["non-object", "missing-policy", "null-policy", "invalid-name"],
)
def test_malformed_environment_response_is_rejected(body: object, code: str) -> None:
    """Malformed or incomplete environment JSON cannot create a policy value."""
    with pytest.raises(VerificationError) as caught:
        environment(ApiResponse(200, {}, body))

    assert caught.value.code == code
