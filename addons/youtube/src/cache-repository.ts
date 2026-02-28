import type { Database as DatabaseType, Statement } from 'better-sqlite3';

export interface YouTubeCacheRow {
	video_id: string;
	data: string;
	fetched_at: string;
}

const STALE_DAYS = 7;

export class YouTubeCacheRepository {
	private stmts: {
		get: Statement<[string], YouTubeCacheRow>;
		upsert: Statement<[string, string]>;
		delete: Statement<[string]>;
	};

	constructor(private db: DatabaseType) {
		this.stmts = {
			get: db.prepare('SELECT * FROM youtube_videos WHERE video_id = ?'),
			upsert: db.prepare(`
				INSERT INTO youtube_videos (video_id, data) VALUES (?, ?)
				ON CONFLICT(video_id) DO UPDATE SET data = excluded.data, fetched_at = datetime('now')
			`),
			delete: db.prepare('DELETE FROM youtube_videos WHERE video_id = ?')
		};
	}

	get(videoId: string): YouTubeCacheRow | null {
		return this.stmts.get.get(videoId) ?? null;
	}

	upsert(videoId: string, data: string): void {
		this.stmts.upsert.run(videoId, data);
	}

	delete(videoId: string): boolean {
		return this.stmts.delete.run(videoId).changes > 0;
	}

	isFresh(fetchedAt: string): boolean {
		const fetched = new Date(fetchedAt + 'Z');
		const now = new Date();
		const diffMs = now.getTime() - fetched.getTime();
		const diffDays = diffMs / (1000 * 60 * 60 * 24);
		return diffDays < STALE_DAYS;
	}
}
