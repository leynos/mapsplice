# Roadmap JSON export format

`mapsplice export --format json ROADMAP.md` derives this document from the
canonical Markdown roadmap. The export is a view of the parsed roadmap, not a
second source of truth. `mapsplice export --check EXPORT.json ROADMAP.md`
checks that a committed view is current.

## Versioning

The root `format_version` is `1`. Consumers should check this value before
reading other fields. Any change to the JSON shape or to the meaning of an
existing field requires incrementing `FORMAT_VERSION` and updating this
document. The exporter emits pretty JSON with two-space indentation, fixed
field order, and one final newline.

## Root and item fields

All arrays retain Markdown source order. Numbers are dotted strings. Checkbox
states are `open` or `done`. Fields are always present, including fields whose
value is `null` or an empty array.

| Object | Fields                                                                    |
| ------ | ------------------------------------------------------------------------- |
| Root   | `format_version`, `phases`                                                |
| Phase  | `number`, `title`, `entries`, `requires`, `steps`, `trailing`             |
| Step   | `number`, `title`, `question`, `entries`, `requires`, `tasks`, `trailing` |
| Task   | `number`, `title`, `state`, `requires`, `entries`, `details`              |
| Detail | `number`, `title`, `state`, `requires`, `entries`, `details`              |
| Entry  | `key`, `value`, `entries`                                                 |

`question` is the first unlabelled step paragraph, or `null` when there is no
such paragraph. A detail's `number` is its dotted structural sub-task number;
an unnumbered checkbox detail uses `null`. An entry's `key` is a detected
label, or `null` for unlabelled content. `entries` and `details` are ordered
arrays and may be empty.

Phase `entries` represent content before the first step. Phase `trailing`
represents content after the last step. Step `entries` represent content before
its task list other than the question, and step `trailing` represents content
after the task list. Task and detail entries represent their own paragraphs and
non-checkbox bullets. Numbered addendum sub-tasks and nested checkbox bullets
appear in `details` in source order.

## Text and labels

Text is rendered from supported inline Markdown nodes, preserving code spans,
emphasis and links. Whitespace runs, including CRLF line endings and Markdown
hard breaks, collapse to one ASCII space; leading and trailing whitespace is
removed. Formatting and whitespace changes in the Markdown therefore do not
create semantic export drift.

A leading label is a short phrase of ASCII letters, spaces or hyphens ending in
`:`, or a leading strong-emphasis node containing such a phrase and colon. The
label becomes `key`; the remaining paragraph text becomes `value`. A following
bullet list belongs to an empty labelled entry, for example `See:`. The
document preamble and source file path are intentionally excluded.

## Dependencies

Each item's `requires` array contains dotted roadmap-anchor strings found in
that item's title and own content. It preserves every anchor level accepted by
the existing `Requires` rewriter, including phases, steps, tasks and structural
sub-tasks. A requirement belongs to the nearest numbered task, structural
sub-task or checkbox detail that contains it. Dependencies are deduplicated in
first-occurrence order. Nested checkbox content belongs to its detail and is
omitted from the parent item's dependency scan.

The exporter uses the same `Requires` candidate scanner as the Markdown
rewriter. It recognizes hard-wrapped clauses, excludes inline and block code,
HTML, link destinations and section references beginning with `§`, and keeps
semicolon and non-numeric period clause boundaries. A malformed anchor-like
token or unresolved roadmap reference is an error with a source line and
column. Nested `Requires` bullets are handled by the same traversal as other
task-body content.

## Strictness

The exporter rejects unsupported or malformed structures instead of omitting
them. It requires checkboxes on tasks and details. Task bodies accept
paragraphs and unordered bullets, including numbered structural sub-tasks and
checkbox details. Code blocks, tables, HTML, block quotes, ordered lists,
unsupported inline nodes, and malformed numbered checkbox prefixes fail with a
source location. Phase and step lists are accepted only beneath an empty
labelled paragraph. The preamble is parsed for Markdown validity but is not
included in the JSON document.

## Example

```json
{
  "format_version": 1,
  "phases": [
    {
      "number": "1",
      "title": "Planning",
      "entries": [
        {
          "key": "Idea",
          "value": "Keep Markdown canonical.",
          "entries": []
        }
      ],
      "requires": [],
      "steps": [
        {
          "number": "1.1",
          "title": "Export the roadmap",
          "question": "Can downstream tools read the roadmap?",
          "entries": [],
          "requires": [],
          "tasks": [
            {
              "number": "1.1.1",
              "title": "Add deterministic JSON output",
              "state": "open",
              "requires": [],
              "entries": [],
              "details": [
                {
                  "number": null,
                  "title": "Compare committed output",
                  "state": "open",
                  "requires": [],
                  "entries": [],
                  "details": []
                }
              ]
            }
          ],
          "trailing": []
        }
      ],
      "trailing": []
    }
  ]
}
```
