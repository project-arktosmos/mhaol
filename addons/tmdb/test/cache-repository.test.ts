import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import Database from 'better-sqlite3';
import { TmdbCacheRepository } from '../src/cache-repository.js';

const TMDB_SCHEMA_SQL = `
CREATE TABLE IF NOT EXISTS tmdb_movies (
    tmdb_id INTEGER PRIMARY KEY,
    data TEXT NOT NULL,
    fetched_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS tmdb_tv_shows (
    tmdb_id INTEGER PRIMARY KEY,
    data TEXT NOT NULL,
    fetched_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS tmdb_seasons (
    tmdb_id INTEGER NOT NULL,
    season_number INTEGER NOT NULL,
    data TEXT NOT NULL,
    fetched_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (tmdb_id, season_number)
);
`;

describe('TmdbCacheRepository', () => {
	let db: InstanceType<typeof Database>;
	let repo: TmdbCacheRepository;

	beforeEach(() => {
		db = new Database(':memory:');
		db.exec(TMDB_SCHEMA_SQL);
		repo = new TmdbCacheRepository(db);
	});

	afterEach(() => {
		db?.close();
	});

	// Movie cache

	describe('movies', () => {
		const movieData = JSON.stringify({ id: 550, title: 'Fight Club' });

		it('should return null for non-existent movie', () => {
			expect(repo.getMovie(550)).toBeNull();
		});

		it('should upsert and retrieve a movie', () => {
			repo.upsertMovie(550, movieData);
			const row = repo.getMovie(550);
			expect(row).not.toBeNull();
			expect(row!.tmdb_id).toBe(550);
			expect(row!.data).toBe(movieData);
			expect(row!.fetched_at).toBeDefined();
		});

		it('should update data on upsert', () => {
			repo.upsertMovie(550, movieData);
			const updated = JSON.stringify({ id: 550, title: 'Fight Club', tagline: 'Updated' });
			repo.upsertMovie(550, updated);
			const row = repo.getMovie(550);
			expect(row!.data).toBe(updated);
		});

		it('should delete a movie', () => {
			repo.upsertMovie(550, movieData);
			expect(repo.deleteMovie(550)).toBe(true);
			expect(repo.getMovie(550)).toBeNull();
		});

		it('should return false when deleting non-existent movie', () => {
			expect(repo.deleteMovie(999)).toBe(false);
		});
	});

	// TV show cache

	describe('tv shows', () => {
		const tvData = JSON.stringify({ id: 1396, name: 'Breaking Bad' });

		it('should return null for non-existent TV show', () => {
			expect(repo.getTvShow(1396)).toBeNull();
		});

		it('should upsert and retrieve a TV show', () => {
			repo.upsertTvShow(1396, tvData);
			const row = repo.getTvShow(1396);
			expect(row).not.toBeNull();
			expect(row!.tmdb_id).toBe(1396);
			expect(row!.data).toBe(tvData);
		});

		it('should update data on upsert', () => {
			repo.upsertTvShow(1396, tvData);
			const updated = JSON.stringify({ id: 1396, name: 'Breaking Bad', status: 'Ended' });
			repo.upsertTvShow(1396, updated);
			const row = repo.getTvShow(1396);
			expect(row!.data).toBe(updated);
		});

		it('should delete a TV show', () => {
			repo.upsertTvShow(1396, tvData);
			expect(repo.deleteTvShow(1396)).toBe(true);
			expect(repo.getTvShow(1396)).toBeNull();
		});
	});

	// Season cache

	describe('seasons', () => {
		const seasonData = JSON.stringify({ id: 1, name: 'Season 1', episodes: [] });

		it('should return null for non-existent season', () => {
			expect(repo.getSeason(1396, 1)).toBeNull();
		});

		it('should upsert and retrieve a season', () => {
			repo.upsertSeason(1396, 1, seasonData);
			const row = repo.getSeason(1396, 1);
			expect(row).not.toBeNull();
			expect(row!.tmdb_id).toBe(1396);
			expect(row!.season_number).toBe(1);
			expect(row!.data).toBe(seasonData);
		});

		it('should store multiple seasons for the same show', () => {
			const s1 = JSON.stringify({ name: 'Season 1' });
			const s2 = JSON.stringify({ name: 'Season 2' });
			repo.upsertSeason(1396, 1, s1);
			repo.upsertSeason(1396, 2, s2);

			expect(repo.getSeason(1396, 1)!.data).toBe(s1);
			expect(repo.getSeason(1396, 2)!.data).toBe(s2);
		});

		it('should update data on upsert', () => {
			repo.upsertSeason(1396, 1, seasonData);
			const updated = JSON.stringify({ id: 1, name: 'Season 1', episodes: [{}] });
			repo.upsertSeason(1396, 1, updated);
			expect(repo.getSeason(1396, 1)!.data).toBe(updated);
		});

		it('should delete a specific season', () => {
			repo.upsertSeason(1396, 1, seasonData);
			repo.upsertSeason(1396, 2, seasonData);
			expect(repo.deleteSeason(1396, 1)).toBe(true);
			expect(repo.getSeason(1396, 1)).toBeNull();
			expect(repo.getSeason(1396, 2)).not.toBeNull();
		});

		it('should delete all seasons for a show', () => {
			repo.upsertSeason(1396, 1, seasonData);
			repo.upsertSeason(1396, 2, seasonData);
			repo.upsertSeason(1396, 3, seasonData);
			const deleted = repo.deleteSeasonsByShow(1396);
			expect(deleted).toBe(3);
			expect(repo.getSeason(1396, 1)).toBeNull();
			expect(repo.getSeason(1396, 2)).toBeNull();
			expect(repo.getSeason(1396, 3)).toBeNull();
		});
	});

	// Freshness checks

	describe('isFresh', () => {
		it('should return true for a recent timestamp', () => {
			const now = new Date().toISOString().replace('T', ' ').replace('Z', '').split('.')[0];
			expect(repo.isFresh(now)).toBe(true);
		});

		it('should return false for an old timestamp', () => {
			const old = new Date(Date.now() - 8 * 24 * 60 * 60 * 1000)
				.toISOString()
				.replace('T', ' ')
				.replace('Z', '')
				.split('.')[0];
			expect(repo.isFresh(old)).toBe(false);
		});

		it('should return true for timestamp less than 7 days old', () => {
			const sixDaysAgo = new Date(Date.now() - 6 * 24 * 60 * 60 * 1000)
				.toISOString()
				.replace('T', ' ')
				.replace('Z', '')
				.split('.')[0];
			expect(repo.isFresh(sixDaysAgo)).toBe(true);
		});
	});
});
