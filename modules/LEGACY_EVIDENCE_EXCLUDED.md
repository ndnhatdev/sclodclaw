# Legacy Modules Tree - Evidence Excluded

This `modules/*` root tree is legacy/transitional content and is excluded from
Cut 02 to Cut 04 active implementation evidence.

Active evidence paths for current refactor execution:

- `src/core/*`
- `src/modules/*/*`

Rules:

- Do not use `modules/*` root files as acceptance evidence for Cut 02 to Cut 04.
- Do not wire lifecycle host discovery to this tree for current evidence runs.
- Keep this tree only as migration reference until canonical cleanup removes or
  archives it in a dedicated follow-up cut.
