<h1 align="center">
  <img src="./images/minas.png" style="width: 25%; height: auto;">
</h1>

# Minas Tirith

Minas Tirith is a terminal-first, offline-capable personal reference manager for books, articles, theses, reports, and miscellaneous documents. It stores your library locally in SQLite, fetches metadata from several bibliographic providers so you don't have to type it in by hand, and lets you share collections directly with other peers over a peer-to-peer network — no server required.

## Features

**Reference management**

- Add references by picking a local `.pdf` or `.epub` file from an in-app file explorer
- Review and edit fetched metadata before saving, or edit it later
- Organize items into collections; filter the item list by collection and by item type
- Export a single item, or an entire collection, as BibTeX to the system clipboard
- Cover art shown inline in the terminal: cached/downloaded covers, or generated locally
  (PDF: first page via `pdftoppm`; EPUB: embedded cover via the `epub` crate)
- Auto-fetches missing abstracts for the selected item when a provider can supply one

**Metadata fetching**

Candidates are pulled from multiple providers and merged/deduplicated before you pick one:

| Provider          | Data                                                              | Requires API key |
| ------------------ | ------------------------------------------------------------------ | :---------------: |
| Open Library       | title, authors, year, ISBN, cover URL                              |         no        |
| Crossref            | title, authors, DOI, publication date, inferred item type          |         no        |
| OpenAlex            | title, authors, DOI, publication date, source venue, concepts as tags |      no        |
| Semantic Scholar    | title, authors, DOI, publication date, venue                       |         no        |
| Google Books        | title, authors, DOI, publisher, categories as tags                 |        yes        |
| CORE                | title, authors, DOI, publication date, field of study, publisher   |        yes        |

**Peer-to-peer library sharing**

Built on [iroh](https://iroh.computer) — no accounts, no central server:

- Publish a collection as a shared library; get back an invite ticket to hand out
- Subscribe to someone else's shared library using their ticket
- Browse a subscribed library's papers and import individual items into your own archive
- Manage your own published libraries: regenerate/copy tickets, revoke a publication
- Subscriptions stay live — updates from the owner sync in the background

## Project layout

This is a Cargo workspace:

| Crate                | What it is                                                          |
| --------------------- | ---------------------------------------------------------------------- |
| `minastirith-core`     | Application/domain logic: database access, metadata providers, p2p sharing, shared state — no UI code |
| `minastirith-core-derive` | Proc macros used by `core` (`Selectable`, `Focusable`, `Cyclable`)  |
| `minastirith-tui`      | The terminal UI (binary: `minastirith`) — the primary, usable front-end |
| `minastirith-gui`      | A graphical front-end (binary: `minastirith-app`) — early scaffold, not yet functional |

## Keybindings

### Normal mode — items focus

| Key          | Action                              |
| ------------ | ------------------------------------ |
| `j` / `Down` | Select next item                     |
| `k` / `Up`   | Select previous item                 |
| `[` / `]`    | Previous / next item type tab        |
| `a`          | Open file explorer to add a new item |
| `e`          | Edit selected item metadata          |
| `b`          | Copy selected item as BibTeX         |
| `c`          | Assign selected item to collections  |
| `L`          | Browse subscribed shared libraries   |
| `Enter`      | Open selected file                   |
| `Tab`        | Switch focus to collections          |
| `q`          | Quit                                 |

### Normal mode — collections focus

| Key          | Action                                     |
| ------------ | -------------------------------------------- |
| `j` / `Down` | Select next collection                       |
| `k` / `Up`   | Select previous collection                   |
| `n`          | Create a new collection                      |
| `d`          | Delete selected collection                   |
| `c`          | Assign items to selected collection          |
| `b`          | Copy BibTeX for the collection's items       |
| `p`          | Publish selected collection as a shared library |
| `L`          | Manage your published shared libraries       |
| `Enter`      | Use collection as active list filter         |
| `Tab`        | Switch focus to items                        |
| `q`          | Quit                                         |

`?` opens the in-app help screen from anywhere in normal mode, listing every binding below.

### Popups

| Popup               | Keys                                                       |
| -------------------- | ------------------------------------------------------------ |
| File explorer         | `a` fetch metadata for file · `Esc`/`Backspace`/`q` back      |
| Metadata selection    | `j`/`k` navigate · `Enter` open edit form · `Esc`/`q` cancel  |
| Metadata edit         | `j`/`k` navigate fields · `Enter` toggle editing · `t` cycle item type · `Ctrl+S` save · `Esc`/`q` cancel |
| Collection create      | `Enter` create · `Esc` cancel                                 |
| Collection assign      | `j`/`k` navigate · `Space`/`Enter` toggle · `Ctrl+S` save · `Esc`/`q` cancel |
| Library publish       | `Tab` switch field · `Ctrl+S` publish & generate ticket · `Esc` cancel |
| Library subscribe     | `Tab` switch field · `Ctrl+S` subscribe using ticket · `Esc` cancel |
| Library browse        | `Tab` switch subscriptions/papers · `j`/`k` navigate · `a` add subscription · `d` remove subscription · `Enter` import paper · `Esc`/`q` close |
| Library manage        | `j`/`k` navigate · `t` generate/refresh ticket · `c` copy ticket · `d` delete publication · `Esc`/`q` close |
| Help                  | `j`/`k` scroll · `Esc`/`q` close                              |

## Build and run

Run the TUI (the primary, usable front-end):

```bash
cargo run --bin minastirith
```

Build release binaries:

```bash
cargo build --release
```

Install the TUI locally:

```bash
cargo install --path crates/tui
```

## Configuration

Configuration is read from `<config_dir>/config.toml`, overridable with `MINASTIRITH_`-prefixed environment variables.

```toml
[api_keys]
google_books = "..."
core = "..."

ephemeral_identity = false
```

- `api_keys` — provider name → API key, for providers that require one (Google Books, CORE)
- `ephemeral_identity` — when `true`, generates a fresh p2p identity every run instead of persisting one to disk (useful for testing)

## Requirements

- Rust toolchain
- `pdftoppm` available on `PATH` for PDF cover generation (Poppler)
- A terminal/protocol combination supported by `ratatui-image` for inline cover rendering

## Storage paths

Data and cache directories are resolved via `directories::ProjectDirs("com", "TheSpanishInquisition", "minastirith")`.

- Database: `<data_dir>/minastirith.db`
- Cover cache: `<cache_dir>/covers/`
- P2P share identity key: `<data_dir>/share_identity.key`
- Config file: `<config_dir>/config.toml`

Database migrations run automatically at startup via `sqlx::migrate!()`.

## Database shape

```
items ──┬── item_authors ──── authors
        ├── item_tags ─────── tags
        └── item_collections ─ collections ── shared_libraries
                                                     │
                                       library_subscriptions
```

`shared_libraries` tracks collections you've published; `library_subscriptions` tracks libraries you follow from other peers.

## Status

This repository is in **active development**.

The local archive workflow is usable end-to-end: add/edit/open items, fetch metadata, organize collections, export BibTeX, cache covers, all backed by SQLite.

Peer-to-peer library sharing (publish/subscribe/browse/import) is implemented and wired up, but still being refined for reliability and UX.

Known limitations:

- Search mode is unfinished (`/` enters an unimplemented UI path)
- The GUI crate (`minastirith-gui`) is an early scaffold, not yet usable
- P2P sharing UX and reliability are still evolving
