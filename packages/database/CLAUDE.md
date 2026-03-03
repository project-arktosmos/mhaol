# Package: database

**Location:** `packages/database/`
**Database:** SQLite via `better-sqlite3`
**Testing:** Vitest

## Source Structure

```
src/
├── index.ts              # Main exports
├── connection.ts         # Database connection setup
├── schema.ts             # Full schema as raw SQL (CREATE TABLE statements)
├── types.ts              # Shared TypeScript types
├── utils/                # Database utilities
└── repositories/         # Repository classes (one per table/domain)
    ├── index.ts
    ├── category.repository.ts
    ├── image-tag.repository.ts
    ├── library.repository.ts
    ├── library-item.repository.ts
    ├── library-item-link.repository.ts
    ├── link-source.repository.ts
    ├── media-type.repository.ts
    ├── metadata.repository.ts
    ├── settings.repository.ts
    ├── torrent-download.repository.ts
    └── youtube-download.repository.ts
```

## Exports

The package exposes three entry points:

```typescript
import { ... } from 'database';              // src/index.ts — connection, schema
import { ... } from 'database/repositories'; // src/repositories/index.ts — all repos
import type { ... } from 'database/types';   // src/types.ts — shared types
```

## Schema

The full database schema is defined in `src/schema.ts` as raw SQL strings. Tables use `CREATE TABLE IF NOT EXISTS` for idempotent initialization.

Key tables: `settings`, `metadata`, `media_types`, `categories`, `libraries`, `library_items`, `library_item_links`, `link_sources`, `youtube_downloads`, `torrent_downloads`, `image_tags`

## Repository Pattern

Each repository wraps a `better-sqlite3` database instance and provides typed CRUD operations:

```typescript
// Example: creating a new repository
export class MyRepository {
    constructor(private db: Database) {}

    getAll(): MyItem[] {
        return this.db.prepare('SELECT * FROM my_table').all() as MyItem[];
    }

    insert(item: MyItem): void {
        this.db.prepare('INSERT INTO my_table (id, name) VALUES (?, ?)').run(item.id, item.name);
    }
}
```

## Testing

```bash
pnpm --filter database test    # or from root: pnpm test
```

Tests live in `test/` mirroring `src/repositories/`.
