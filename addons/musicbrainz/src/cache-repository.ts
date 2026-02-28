import type { Database as DatabaseType, Statement } from 'better-sqlite3';

export interface MusicBrainzCacheRow {
	mbid: string;
	data: string;
	fetched_at: string;
}

const STALE_DAYS = 7;

export class MusicBrainzCacheRepository {
	private stmts: {
		getArtist: Statement<[string], MusicBrainzCacheRow>;
		upsertArtist: Statement<[string, string]>;
		deleteArtist: Statement<[string]>;
		getReleaseGroup: Statement<[string], MusicBrainzCacheRow>;
		upsertReleaseGroup: Statement<[string, string]>;
		deleteReleaseGroup: Statement<[string]>;
		getRecording: Statement<[string], MusicBrainzCacheRow>;
		upsertRecording: Statement<[string, string]>;
		deleteRecording: Statement<[string]>;
	};

	constructor(private db: DatabaseType) {
		this.stmts = {
			getArtist: db.prepare('SELECT * FROM musicbrainz_artists WHERE mbid = ?'),
			upsertArtist: db.prepare(`
				INSERT INTO musicbrainz_artists (mbid, data) VALUES (?, ?)
				ON CONFLICT(mbid) DO UPDATE SET data = excluded.data, fetched_at = datetime('now')
			`),
			deleteArtist: db.prepare('DELETE FROM musicbrainz_artists WHERE mbid = ?'),
			getReleaseGroup: db.prepare('SELECT * FROM musicbrainz_release_groups WHERE mbid = ?'),
			upsertReleaseGroup: db.prepare(`
				INSERT INTO musicbrainz_release_groups (mbid, data) VALUES (?, ?)
				ON CONFLICT(mbid) DO UPDATE SET data = excluded.data, fetched_at = datetime('now')
			`),
			deleteReleaseGroup: db.prepare('DELETE FROM musicbrainz_release_groups WHERE mbid = ?'),
			getRecording: db.prepare('SELECT * FROM musicbrainz_recordings WHERE mbid = ?'),
			upsertRecording: db.prepare(`
				INSERT INTO musicbrainz_recordings (mbid, data) VALUES (?, ?)
				ON CONFLICT(mbid) DO UPDATE SET data = excluded.data, fetched_at = datetime('now')
			`),
			deleteRecording: db.prepare('DELETE FROM musicbrainz_recordings WHERE mbid = ?')
		};
	}

	getArtist(mbid: string): MusicBrainzCacheRow | null {
		return this.stmts.getArtist.get(mbid) ?? null;
	}

	upsertArtist(mbid: string, data: string): void {
		this.stmts.upsertArtist.run(mbid, data);
	}

	deleteArtist(mbid: string): boolean {
		return this.stmts.deleteArtist.run(mbid).changes > 0;
	}

	getReleaseGroup(mbid: string): MusicBrainzCacheRow | null {
		return this.stmts.getReleaseGroup.get(mbid) ?? null;
	}

	upsertReleaseGroup(mbid: string, data: string): void {
		this.stmts.upsertReleaseGroup.run(mbid, data);
	}

	deleteReleaseGroup(mbid: string): boolean {
		return this.stmts.deleteReleaseGroup.run(mbid).changes > 0;
	}

	getRecording(mbid: string): MusicBrainzCacheRow | null {
		return this.stmts.getRecording.get(mbid) ?? null;
	}

	upsertRecording(mbid: string, data: string): void {
		this.stmts.upsertRecording.run(mbid, data);
	}

	deleteRecording(mbid: string): boolean {
		return this.stmts.deleteRecording.run(mbid).changes > 0;
	}

	isFresh(fetchedAt: string): boolean {
		const fetched = new Date(fetchedAt + 'Z');
		const now = new Date();
		const diffMs = now.getTime() - fetched.getTime();
		const diffDays = diffMs / (1000 * 60 * 60 * 24);
		return diffDays < STALE_DAYS;
	}
}
