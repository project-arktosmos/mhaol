import type { Database as DatabaseType, Statement } from 'better-sqlite3';

export interface TmdbCacheRow {
	tmdb_id: number;
	data: string;
	fetched_at: string;
}

export interface TmdbSeasonCacheRow {
	tmdb_id: number;
	season_number: number;
	data: string;
	fetched_at: string;
}

const STALE_DAYS = 7;

export class TmdbCacheRepository {
	private stmts: {
		getMovie: Statement<[number], TmdbCacheRow>;
		upsertMovie: Statement<[number, string]>;
		deleteMovie: Statement<[number]>;
		getTvShow: Statement<[number], TmdbCacheRow>;
		upsertTvShow: Statement<[number, string]>;
		deleteTvShow: Statement<[number]>;
		getSeason: Statement<[number, number], TmdbSeasonCacheRow>;
		upsertSeason: Statement<[number, number, string]>;
		deleteSeason: Statement<[number, number]>;
		deleteSeasonsByShow: Statement<[number]>;
	};

	constructor(private db: DatabaseType) {
		this.stmts = {
			getMovie: db.prepare('SELECT * FROM tmdb_movies WHERE tmdb_id = ?'),
			upsertMovie: db.prepare(`
				INSERT INTO tmdb_movies (tmdb_id, data) VALUES (?, ?)
				ON CONFLICT(tmdb_id) DO UPDATE SET data = excluded.data, fetched_at = datetime('now')
			`),
			deleteMovie: db.prepare('DELETE FROM tmdb_movies WHERE tmdb_id = ?'),
			getTvShow: db.prepare('SELECT * FROM tmdb_tv_shows WHERE tmdb_id = ?'),
			upsertTvShow: db.prepare(`
				INSERT INTO tmdb_tv_shows (tmdb_id, data) VALUES (?, ?)
				ON CONFLICT(tmdb_id) DO UPDATE SET data = excluded.data, fetched_at = datetime('now')
			`),
			deleteTvShow: db.prepare('DELETE FROM tmdb_tv_shows WHERE tmdb_id = ?'),
			getSeason: db.prepare(
				'SELECT * FROM tmdb_seasons WHERE tmdb_id = ? AND season_number = ?'
			),
			upsertSeason: db.prepare(`
				INSERT INTO tmdb_seasons (tmdb_id, season_number, data) VALUES (?, ?, ?)
				ON CONFLICT(tmdb_id, season_number) DO UPDATE SET data = excluded.data, fetched_at = datetime('now')
			`),
			deleteSeason: db.prepare(
				'DELETE FROM tmdb_seasons WHERE tmdb_id = ? AND season_number = ?'
			),
			deleteSeasonsByShow: db.prepare('DELETE FROM tmdb_seasons WHERE tmdb_id = ?')
		};
	}

	getMovie(tmdbId: number): TmdbCacheRow | null {
		return this.stmts.getMovie.get(tmdbId) ?? null;
	}

	upsertMovie(tmdbId: number, data: string): void {
		this.stmts.upsertMovie.run(tmdbId, data);
	}

	deleteMovie(tmdbId: number): boolean {
		return this.stmts.deleteMovie.run(tmdbId).changes > 0;
	}

	getTvShow(tmdbId: number): TmdbCacheRow | null {
		return this.stmts.getTvShow.get(tmdbId) ?? null;
	}

	upsertTvShow(tmdbId: number, data: string): void {
		this.stmts.upsertTvShow.run(tmdbId, data);
	}

	deleteTvShow(tmdbId: number): boolean {
		return this.stmts.deleteTvShow.run(tmdbId).changes > 0;
	}

	getSeason(tmdbId: number, seasonNumber: number): TmdbSeasonCacheRow | null {
		return this.stmts.getSeason.get(tmdbId, seasonNumber) ?? null;
	}

	upsertSeason(tmdbId: number, seasonNumber: number, data: string): void {
		this.stmts.upsertSeason.run(tmdbId, seasonNumber, data);
	}

	deleteSeason(tmdbId: number, seasonNumber: number): boolean {
		return this.stmts.deleteSeason.run(tmdbId, seasonNumber).changes > 0;
	}

	deleteSeasonsByShow(tmdbId: number): number {
		return this.stmts.deleteSeasonsByShow.run(tmdbId).changes;
	}

	isFresh(fetchedAt: string): boolean {
		const fetched = new Date(fetchedAt + 'Z');
		const now = new Date();
		const diffMs = now.getTime() - fetched.getTime();
		const diffDays = diffMs / (1000 * 60 * 60 * 24);
		return diffDays < STALE_DAYS;
	}
}
