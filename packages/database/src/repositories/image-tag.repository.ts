import type { Database as DatabaseType, Statement } from "better-sqlite3";
import type { ImageTagRow } from "../types.js";

type InsertRow = Omit<ImageTagRow, "created_at">;

export class ImageTagRepository {
  private stmts: {
    getByItem: Statement<[string], ImageTagRow>;
    insert: Statement<[InsertRow]>;
    deleteByItem: Statement<[string]>;
    deleteOne: Statement<[string, string]>;
    searchByTag: Statement<[string], ImageTagRow>;
  };

  private replaceForItemTx: ReturnType<DatabaseType["transaction"]>;

  constructor(private db: DatabaseType) {
    this.stmts = {
      getByItem: db.prepare(
        "SELECT * FROM image_tags WHERE library_item_id = ? ORDER BY score DESC",
      ),
      insert: db.prepare(`
				INSERT INTO image_tags (id, library_item_id, tag, score)
				VALUES (@id, @library_item_id, @tag, @score)
			`),
      deleteByItem: db.prepare(
        "DELETE FROM image_tags WHERE library_item_id = ?",
      ),
      deleteOne: db.prepare(
        "DELETE FROM image_tags WHERE library_item_id = ? AND tag = ?",
      ),
      searchByTag: db.prepare(
        "SELECT * FROM image_tags WHERE tag = ? ORDER BY score DESC",
      ),
    };

    this.replaceForItemTx = db.transaction(
      (libraryItemId: string, tags: InsertRow[]) => {
        this.stmts.deleteByItem.run(libraryItemId);
        for (const tag of tags) {
          this.stmts.insert.run(tag);
        }
      },
    );
  }

  getByItem(libraryItemId: string): ImageTagRow[] {
    return this.stmts.getByItem.all(libraryItemId);
  }

  getByItems(libraryItemIds: string[]): Record<string, ImageTagRow[]> {
    const result: Record<string, ImageTagRow[]> = {};
    for (const id of libraryItemIds) {
      result[id] = this.stmts.getByItem.all(id);
    }
    return result;
  }

  replaceForItem(libraryItemId: string, tags: InsertRow[]): void {
    this.replaceForItemTx(libraryItemId, tags);
  }

  addTag(libraryItemId: string, tag: string, score: number): void {
    this.stmts.insert.run({
      id: crypto.randomUUID(),
      library_item_id: libraryItemId,
      tag,
      score
    });
  }

  deleteTag(libraryItemId: string, tag: string): void {
    this.stmts.deleteOne.run(libraryItemId, tag);
  }

  deleteByItem(libraryItemId: string): void {
    this.stmts.deleteByItem.run(libraryItemId);
  }

  searchByTag(tag: string): ImageTagRow[] {
    return this.stmts.searchByTag.all(tag);
  }
}
