# Skill: New Full-Stack Feature

Scaffold a new feature across the entire stack (database, backend, frontend).

## Instructions

1. Ask the user for:
   - Feature/entity name (e.g. "playlists", "bookmarks")
   - Whether it needs a new database table
   - What API endpoints are needed

2. **Database** (if new table needed):
   - Add table to `packages/database/src/schema.ts` (raw SQL, `CREATE TABLE IF NOT EXISTS`)
   - Create repository in `packages/database/src/repositories/{feature}.repository.ts`
   - Export from `packages/database/src/repositories/index.ts`
   - Run tests: `pnpm --filter database test`

3. **Backend**:
   - Create `packages/backend/src/api/{feature}.rs`
   - Implement `pub fn router() -> Router<AppState>`
   - Add `pub mod {feature};` to `src/api/mod.rs`
   - Register in `build_router()`: `.nest("/api/{feature}", {feature}::router())`
   - If new database access is needed, add repo to `AppState` in `src/lib.rs`
   - Run checks: `cargo check -p mhaol-backend && cargo test -p mhaol-backend`

4. **Frontend**:
   - Define types in `packages/mhaol-video/src/types/{feature}.type.ts`
   - Create adapter in `packages/mhaol-video/src/adapters/classes/{feature}.adapter.ts`
   - Create service in `packages/mhaol-video/src/services/{feature}.service.ts`
   - Create component directory `packages/mhaol-video/src/components/{feature}/`
   - Follow all component rules: no `<style>` tags, no inline styles, use `classnames`, TypeScript `$props()` with inline types, callback props for parent communication

5. **Verify**:
   - Run `pnpm lint && pnpm check && pnpm build && pnpm test`
   - Fix any errors before committing

6. **Commit** each logical step separately (schema, backend, frontend) with short English messages.
