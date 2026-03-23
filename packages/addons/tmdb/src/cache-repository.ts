import type { Database as DatabaseType } from 'better-sqlite3';
import { AddonCacheRepository } from '../../common/src/addon-cache.js';
import type { CacheRow } from '../../common/src/addon-cache.js';

const SERVICE = 'tmdb';

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

function toTmdbRow(row: CacheRow): TmdbCacheRow {
	return {
		tmdb_id: Number(row.cache_key),
		data: row.data,
		fetched_at: row.fetched_at
	};
}

function toSeasonRow(row: CacheRow, seasonNumber: number): TmdbSeasonCacheRow {
	return {
		tmdb_id: Number(row.cache_key.split(':')[0]),
		season_number: seasonNumber,
		data: row.data,
		fetched_at: row.fetched_at
	};
}

export class TmdbCacheRepository {
	private cache: AddonCacheRepository;

	constructor(db: DatabaseType) {
		this.cache = new AddonCacheRepository(db);
	}

	getMovie(tmdbId: number): TmdbCacheRow | null {
		const row = this.cache.get(SERVICE, 'movie', String(tmdbId));
		return row ? toTmdbRow(row) : null;
	}

	upsertMovie(tmdbId: number, data: string): void {
		this.cache.upsert(SERVICE, 'movie', String(tmdbId), data);
	}

	deleteMovie(tmdbId: number): boolean {
		return this.cache.delete(SERVICE, 'movie', String(tmdbId));
	}

	getTvShow(tmdbId: number): TmdbCacheRow | null {
		const row = this.cache.get(SERVICE, 'tv', String(tmdbId));
		return row ? toTmdbRow(row) : null;
	}

	upsertTvShow(tmdbId: number, data: string): void {
		this.cache.upsert(SERVICE, 'tv', String(tmdbId), data);
	}

	deleteTvShow(tmdbId: number): boolean {
		return this.cache.delete(SERVICE, 'tv', String(tmdbId));
	}

	getSeason(tmdbId: number, seasonNumber: number): TmdbSeasonCacheRow | null {
		const row = this.cache.get(SERVICE, 'season', `${tmdbId}:${seasonNumber}`);
		return row ? toSeasonRow(row, seasonNumber) : null;
	}

	upsertSeason(tmdbId: number, seasonNumber: number, data: string): void {
		this.cache.upsert(SERVICE, 'season', `${tmdbId}:${seasonNumber}`, data);
	}

	deleteSeason(tmdbId: number, seasonNumber: number): boolean {
		return this.cache.delete(SERVICE, 'season', `${tmdbId}:${seasonNumber}`);
	}

	deleteSeasonsByShow(tmdbId: number): number {
		return this.cache.deleteByServiceType(SERVICE, 'season');
	}

	isFresh(fetchedAt: string): boolean {
		return this.cache.isFresh(fetchedAt);
	}
}
