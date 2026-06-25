# Minas Tirith

A personal reference manager for books, articles, academic papers, reports, and theses. automatically fetch metadata from external providers (Open Library, Crossref) and organization through authors, tags, and categories.

## Features

- 📚 Cataloging of sources of different types
- 🔍 Automatic metadata lookup by title from external providers (Open Library, with Crossref coming soon)
- 🗂️ Local storage on SQLite, no external dependencies for saved data
- 🦀 Written in Rust, using `sqlx` for database access

## Database schema

The database is organized around the `items` table, with many-to-many relationships toward `authors`, `tags`, and `categories`:

```
items ──┬── item_authors ──── authors
        ├── item_tags ─────── tags
        └── item_categories ─ categories
```

Migrations are managed via `sqlx-cli` and live in the `migrations/` folder.

## Metadata providers

The project automatically fetches bibliographic metadata starting from a title, through a common trait (`MetadataFetcher`) implemented by type-specific providers:

| Item type | Provider | Status |
|---|---|---|
| `book` | Open Library | ✅ Implemented |
| `article`, `report`, `thesis` | Crossref | 🚧 Coming soon |

Each provider converts the external API's specific JSON response into a common struct (`ItemMetadata`), so the rest of the application doesn't depend on any provider-specific format.

### Installation

```bash
git clone <repository-url>
cd minastirith
cargo build
```

### Database

The SQLite database is automatically created on first run, in a standard OS-dependent path (handled via `directories`):

- **Linux**: `~/.local/share/minastirith/minastirith.db`
- **macOS**: `~/Library/Application Support/com.yourname.minastirith/minastirith.db`
- **Windows**: `%APPDATA%\yourname\minastirith\minastirith.db`

Migrations are applied automatically at startup via `sqlx::migrate!`.

To run migrations manually:

```bash
sqlx migrate run --database-url sqlite://<path-to-db>
```

## Roadmap

- [x] Database schema (items, authors, tags, categories)
- [x] Metadata provider: Open Library (books)
- [ ] Metadata provider: Crossref (papers/articles/theses)
- [ ] User interface
- [ ] Bibliography import/export (BibTeX?)

## License

_To be defined._

---

> ⚠️ Early-stage project, schema and internal APIs may change without notice.
