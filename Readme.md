# Minas Tirith

A personal reference manager for books, articles, academic papers, reports, and theses, with a terminal user interface.
Automatically fetches metadata from Open Library and Crossref, and organizes sources through authors, tags, and categories.

## Features

- 📚 Cataloguing of sources of different types (books, articles, reports, theses)
- 🔍 Automatic metadata lookup by title from Open Library and Crossref
- 🖼️ Cover image display via terminal graphics protocol (Kitty, Sixel, chafa)
- 📁 Add items by selecting `.pdf` or `.epub` files via a file picker

## Terminal UI

The application presents a two-panel layout:

- **Left panel (35%)**: scrollable list of catalogued items
- **Right panel (65%)**: detail view showing title, type, date, DOI, ISBN, and cover image

### Keybindings

| Key         | Action                                  |
| ----------- | --------------------------------------- |
| `j` / `k`   | Navigate items                          |
| `a`         | Open file picker to add a new item      |
| `/`         | Search (stub)                           |
| `q` / `Esc` | Quit / Cancel / Go back                 |
| `Ctrl+S`    | Trigger metadata fetch (in file picker) |
| `Enter`     | Confirm metadata selection              |

### Workflow

1. Press `a` to open the file picker (filtered to `.pdf` and `.epub`)
2. Navigate to a file and press `Ctrl+S` — the app queries both Open Library and Crossref for matching metadata
3. Select the correct metadata entry from the popup list
4. The item is saved to the database and appears in the left panel
5. Select an item to view its details and cover image

## Database schema

The database is organized around the `items` table, with many-to-many relationships toward `authors`, `tags`, and `categories`:

```
items ──┬── item_authors ──── authors
        ├── item_tags ─────── tags
        └── item_categories ─ categories
```

Migrations are managed via `sqlx-cli` and live in the `migrations/` folder.

### Installation

```bash
git clone git@github.com:SpanishInquisition49/Minas-Tirith.git
cd minastirith
cargo build
```

### Database

The SQLite database is automatically created on first run, in a standard OS-dependent path (handled via `directories`):

- **Linux**: `~/.local/share/minastirith/minastirith.db`
- **macOS**: `~/Library/Application Support/com.TheSpanishInquisition.minastirith/minastirith.db`
- **Windows**: `%APPDATA%\TheSpanishInquisition\minastirith\minastirith.db`

Migrations are applied automatically at startup via `sqlx::migrate!`.

To run migrations manually:

```bash
sqlx migrate run --database-url sqlite://<path-to-db>
```

## Roadmap

- [x] Database schema (items, authors, tags, categories)
- [x] Metadata provider: Open Library (books)
- [x] Metadata provider: Crossref (papers/articles/theses)
- [x] Terminal user interface
- [ ] Tags and categories management in the UI
- [ ] Search functionality
- [ ] Bibliography import/export (BibTeX?)

## License

_To be defined._

---

> ⚠️ Early-stage project, schema and internal APIs may change without notice.
