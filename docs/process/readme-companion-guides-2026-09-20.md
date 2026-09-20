# README companion guides (2026-09-20)

Shortened the root [README.md](../../README.md) so the Table of Contents anchors
work, and moved long operator detail under [readme/](./readme/).

## Problem

TOC entries for Architecture, Usage, and API Documentation pointed at headings
that did not exist on the landing page (`#architecture`, `#usage`,
`#api-documentation`), so GitHub links were dead.

## Change

- Added `docs/readme/` with full guides: architecture, installation, usage, api,
  development, testing (+ folder index).
- Root README keeps a short blurb per section and links into those guides.
- **License** heading and everything below it were preserved unchanged.
- `docs/architecture.md` is now a stub pointing at
  `docs/readme/architecture.md`.

## Note

Do not put Cursor / AI attribution in commits or PR text.
