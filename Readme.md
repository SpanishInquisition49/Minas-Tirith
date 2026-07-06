<h1 align="center">
  <img src="./images/minas.png" style="width: 25%; height: auto;">
</h1>

# Minas Tirith

Minas Tirith is a terminal-first personal reference manager for books, articles, reports, theses, and miscellaneous documents.

It stores metadata in SQLite, fetches candidates from Open Library and Crossref, lets you review/edit metadata before saving, and shows document covers directly in the TUI.

## Current capabilities

- Add references by choosing a local `.pdf` or `.epub` file from an in-app file explorer
- Fetch metadata candidates from:
  - Open Library (`title`, authors, year, ISBN when available, cover URL)
  - Crossref (`title`, authors, DOI, publication date, inferred item type)
- Select a candidate, edit metadata, and save to the archive
- Edit metadata for already-saved publications
- Open a saved file from the list
- Display cached/downloaded covers, or generate covers from local files:
  - PDF: first page via `pdftoppm`
  - EPUB: embedded cover via `epub` crate
- Persist items, authors, and tags in SQLite through SQLx migrations

## Keybindings

### Normal mode

| Key          | Action                                    |
| ------------ | ----------------------------------------- |
| `j` / `Down` | Select next item                          |
| `k` / `Up`   | Select previous item                      |
| `a`          | Open file explorer (add flow)             |
| `e`          | Edit selected item metadata               |
| `Enter`      | Open selected file with system opener     |
| `/`          | Enter search mode (currently placeholder) |
| `q`          | Quit                                      |

### Insert mode (file explorer)

| Key                       | Action                                     |
| ------------------------- | ------------------------------------------ |
| `a`                       | Fetch metadata candidates for current file |
| `Esc` / `Backspace` / `q` | Return to normal mode                      |

### Metadata selection popup

| Key          | Action                  |
| ------------ | ----------------------- |
| `j` / `Down` | Next candidate          |
| `k` / `Up`   | Previous candidate      |
| `Enter`      | Open metadata edit form |
| `Esc` / `q`  | Cancel                  |

### Metadata edit popup

| Key          | Action                      |
| ------------ | --------------------------- |
| `j` / `Down` | Next field                  |
| `k` / `Up`   | Previous field              |
| `Enter`      | Toggle field text editing   |
| `t`          | Cycle item type             |
| `Backspace`  | Delete char (while editing) |
| `Ctrl+S`     | Save                        |
| `Esc` / `q`  | Cancel                      |

## Build and run

```bash
cargo run
```

Build release binary:

```bash
cargo build --release
```

Install locally:

```bash
cargo install --path .
```

## Requirements

- Rust toolchain
- `pdftoppm` available in `PATH` for PDF cover generation (Poppler)
- A terminal/protocol combination supported by `ratatui-image` for inline cover rendering

## Storage paths

Data and cache directories are resolved with `directories::ProjectDirs("com", "TheSpanishInquisition", "minastirith")`.

- Database file: `<data_dir>/minastirith.db`
- Cover cache: `<cache_dir>/covers/`

Migrations are applied automatically at startup via `sqlx::migrate!()`.

## Database shape

Main table graph:

```
items ──┬── item_authors ──── authors
        ├── item_tags ─────── tags
        └── item_categories ─ categories
```

Note: categories exist in schema, but category management is not yet exposed in the UI.

## Status

This is an early-stage project and APIs/UX are still evolving.
