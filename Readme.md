<h1 align="center">
  <img src="./images/minas.png" style="width: 25%; height: auto;">
</h1>

# Minas Tirith

Minas Tirith is a terminal-first personal reference manager for books, articles, reports, theses, and miscellaneous documents.

It stores metadata in SQLite, fetches candidates from Open Library, Crossref, and OpenAlex, lets you review/edit metadata before saving, and shows document covers directly in the TUI.

## Current capabilities

- Add references by choosing a local `.pdf` or `.epub` file from an in-app file explorer
- Fetch metadata candidates from:
  - Open Library (`title`, authors, year, ISBN when available, cover URL)
  - Crossref (`title`, authors, DOI, publication date, inferred item type)
  - OpenAlex (`title`, authors, DOI, publication date, source venue, concepts as tags)
- Select a candidate, edit metadata, and save to the archive
- Edit metadata for already-saved publications
- Open a saved file from the list
- Copy the selected item's BibTeX entry to the system clipboard
- Copy BibTeX entries for the selected collection to the system clipboard
- Filter the item list by type and by selected collection
- Create and delete collections
- Assign items to collections (from either the item list or collection view)
- Display cached/downloaded covers, or generate covers from local files:
  - PDF: first page via `pdftoppm`
  - EPUB: embedded cover via `epub` crate
- Auto-fetch missing abstracts for selected items when possible
- Persist items, authors, and tags in SQLite through SQLx migrations

## Keybindings

### Normal mode (items focus)

| Key          | Action                                |
| ------------ | ------------------------------------- |
| `j` / `Down` | Select next item                      |
| `k` / `Up`   | Select previous item                  |
| `[` / `]`    | Switch item type tab                  |
| `a`          | Open file explorer (add flow)         |
| `e`          | Edit selected item metadata           |
| `b`          | Copy selected item as BibTeX          |
| `c`          | Assign selected item to collections   |
| `Enter`      | Open selected file with system opener |
| `Tab`        | Switch focus to collections           |
| `q`          | Quit                                  |

### Normal mode (collections focus)

| Key          | Action                                           |
| ------------ | ------------------------------------------------ |
| `j` / `Down` | Select next collection                           |
| `k` / `Up`   | Select previous collection                       |
| `n`          | Create a new collection                          |
| `d`          | Delete selected collection                       |
| `c`          | Assign items to selected collection              |
| `b`          | Copy BibTeX for selected collection's items      |
| `Enter`      | Use selected collection as active list filter    |
| `Tab`        | Switch focus to items                            |
| `q`          | Quit                                             |

### Insert mode (file explorer popup)

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

### Collection create popup

| Key     | Action                      |
| ------- | --------------------------- |
| `Enter` | Create collection           |
| `Esc`   | Cancel                      |

### Collection assign popup

| Key                    | Action                 |
| ---------------------- | ---------------------- |
| `j` / `Down`           | Next item/collection   |
| `k` / `Up`             | Previous item/collection |
| `Space` / `Enter`      | Toggle assignment      |
| `Ctrl+S`               | Save assignments       |
| `Esc` / `q`            | Cancel                 |

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
        └── item_collections ─ collections
```

## Status

This repository is in **active development**.

The local archive workflow is already usable end-to-end (add/edit/open items, metadata fetch, collections, BibTeX export, cover caching, and SQLite-backed persistence).

Current work is focused on **peer-to-peer sharing** (publishing collections as shared libraries, subscribing via ticket, browsing shared papers, and importing them locally). That area is present in the codebase but still evolving.

Known limitations right now:

- Search mode is still unfinished (`/` enters an unimplemented UI path)
- P2P/library-sharing UX and reliability are still being refined
- UX and data model are still evolving
