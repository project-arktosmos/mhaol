import type { Database as DatabaseType, Statement } from 'better-sqlite3';

export interface LrcLibLyricsRow {
	lrclib_id: number;
	track_name: string;
	artist_name: string;
	album_name: string;
	duration: number;
	instrumental: number;
	plain_lyrics: string | null;
	synced_lyrics: string | null;
	fetched_at: string;
}

export interface LrcLibLookupRow {
	library_item_id: string;
	lrclib_id: number | null;
	status: 'found' | 'not_found';
	looked_up_at: string;
}

export class LrcLibCacheRepository {
	private stmts: {
		getLyrics: Statement<[number], LrcLibLyricsRow>;
		upsertLyrics: Statement<[number, string, string, string, number, number, string | null, string | null]>;
		getLookup: Statement<[string], LrcLibLookupRow>;
		upsertLookup: Statement<[string, number | null, string]>;
		getByLibraryItem: Statement<[string], LrcLibLyricsRow>;
		deleteLookup: Statement<[string]>;
	};

	constructor(private db: DatabaseType) {
		this.stmts = {
			getLyrics: db.prepare('SELECT * FROM lrclib_lyrics WHERE lrclib_id = ?'),
			upsertLyrics: db.prepare(`
				INSERT INTO lrclib_lyrics (lrclib_id, track_name, artist_name, album_name, duration, instrumental, plain_lyrics, synced_lyrics)
				VALUES (?, ?, ?, ?, ?, ?, ?, ?)
				ON CONFLICT(lrclib_id) DO UPDATE SET
					track_name = excluded.track_name,
					artist_name = excluded.artist_name,
					album_name = excluded.album_name,
					duration = excluded.duration,
					instrumental = excluded.instrumental,
					plain_lyrics = excluded.plain_lyrics,
					synced_lyrics = excluded.synced_lyrics,
					fetched_at = datetime('now')
			`),
			getLookup: db.prepare('SELECT * FROM lrclib_lookups WHERE library_item_id = ?'),
			upsertLookup: db.prepare(`
				INSERT INTO lrclib_lookups (library_item_id, lrclib_id, status)
				VALUES (?, ?, ?)
				ON CONFLICT(library_item_id) DO UPDATE SET
					lrclib_id = excluded.lrclib_id,
					status = excluded.status,
					looked_up_at = datetime('now')
			`),
			getByLibraryItem: db.prepare(`
				SELECT l.* FROM lrclib_lyrics l
				INNER JOIN lrclib_lookups k ON k.lrclib_id = l.lrclib_id
				WHERE k.library_item_id = ? AND k.status = 'found'
			`),
			deleteLookup: db.prepare('DELETE FROM lrclib_lookups WHERE library_item_id = ?')
		};
	}

	getLyrics(lrclibId: number): LrcLibLyricsRow | null {
		return this.stmts.getLyrics.get(lrclibId) ?? null;
	}

	upsertLyrics(
		lrclibId: number,
		trackName: string,
		artistName: string,
		albumName: string,
		duration: number,
		instrumental: boolean,
		plainLyrics: string | null,
		syncedLyrics: string | null
	): void {
		this.stmts.upsertLyrics.run(
			lrclibId,
			trackName,
			artistName,
			albumName,
			duration,
			instrumental ? 1 : 0,
			plainLyrics,
			syncedLyrics
		);
	}

	getLookup(libraryItemId: string): LrcLibLookupRow | null {
		return this.stmts.getLookup.get(libraryItemId) ?? null;
	}

	upsertLookup(libraryItemId: string, lrclibId: number | null, status: 'found' | 'not_found'): void {
		this.stmts.upsertLookup.run(libraryItemId, lrclibId, status);
	}

	getByLibraryItem(libraryItemId: string): LrcLibLyricsRow | null {
		return this.stmts.getByLibraryItem.get(libraryItemId) ?? null;
	}

	deleteLookup(libraryItemId: string): boolean {
		return this.stmts.deleteLookup.run(libraryItemId).changes > 0;
	}
}
