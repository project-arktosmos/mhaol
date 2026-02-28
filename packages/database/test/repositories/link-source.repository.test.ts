import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import Database from 'better-sqlite3';
import { initializeSchema } from '../../src/schema.js';
import { LinkSourceRepository } from '../../src/repositories/link-source.repository.js';

describe('LinkSourceRepository', () => {
	let db: InstanceType<typeof Database>;
	let repo: LinkSourceRepository;

	beforeEach(() => {
		db = new Database(':memory:');
		db.pragma('foreign_keys = ON');
		initializeSchema(db);
		repo = new LinkSourceRepository(db);
	});

	afterEach(() => {
		db?.close();
	});

	it('should return empty array when no link sources exist', () => {
		expect(repo.getAll()).toEqual([]);
	});

	it('should upsert and retrieve a link source', () => {
		repo.upsert({ id: 'ls-1', plugin: 'tmdb', service: 'tmdb', label: 'TMDB', media_type_id: 'video', category_id: 'movies' });
		const all = repo.getAll();
		expect(all).toHaveLength(1);
		expect(all[0].plugin).toBe('tmdb');
		expect(all[0].service).toBe('tmdb');
		expect(all[0].label).toBe('TMDB');
		expect(all[0].media_type_id).toBe('video');
		expect(all[0].category_id).toBe('movies');
	});

	it('should update on upsert conflict', () => {
		repo.upsert({ id: 'ls-1', plugin: 'tmdb', service: 'tmdb', label: 'TMDB', media_type_id: 'video', category_id: 'movies' });
		repo.upsert({ id: 'ls-2', plugin: 'tmdb-v2', service: 'tmdb', label: 'TMDB v2', media_type_id: 'video', category_id: 'movies' });
		const all = repo.getAll();
		expect(all).toHaveLength(1);
		expect(all[0].plugin).toBe('tmdb-v2');
		expect(all[0].label).toBe('TMDB v2');
	});

	it('should get link sources by media type (null category only)', () => {
		repo.upsert({ id: 'ls-1', plugin: 'tmdb', service: 'tmdb', label: 'TMDB', media_type_id: 'video', category_id: null });
		repo.upsert({ id: 'ls-2', plugin: 'tmdb', service: 'tmdb', label: 'TMDB Movies', media_type_id: 'video', category_id: 'movies' });
		repo.upsert({ id: 'ls-3', plugin: 'mb', service: 'musicbrainz', label: 'MusicBrainz', media_type_id: 'audio', category_id: null });

		const videoSources = repo.getByMediaType('video');
		expect(videoSources).toHaveLength(1);
		expect(videoSources[0].category_id).toBeNull();

		const audioSources = repo.getByMediaType('audio');
		expect(audioSources).toHaveLength(1);
		expect(audioSources[0].service).toBe('musicbrainz');
	});

	it('should get link sources by media type and category', () => {
		repo.upsert({ id: 'ls-1', plugin: 'tmdb', service: 'tmdb', label: 'TMDB', media_type_id: 'video', category_id: null });
		repo.upsert({ id: 'ls-2', plugin: 'tmdb', service: 'tmdb-movies', label: 'TMDB Movies', media_type_id: 'video', category_id: 'movies' });
		repo.upsert({ id: 'ls-3', plugin: 'tmdb', service: 'tmdb-tv', label: 'TMDB TV', media_type_id: 'video', category_id: 'tv' });

		const movieSources = repo.getByMediaTypeAndCategory('video', 'movies');
		expect(movieSources).toHaveLength(2);
		const services = movieSources.map((s) => s.service).sort();
		expect(services).toEqual(['tmdb', 'tmdb-movies']);

		const tvSources = repo.getByMediaTypeAndCategory('video', 'tv');
		expect(tvSources).toHaveLength(2);

		const ytSources = repo.getByMediaTypeAndCategory('video', 'youtube');
		expect(ytSources).toHaveLength(1);
		expect(ytSources[0].category_id).toBeNull();
	});

	it('should delete all link sources by plugin', () => {
		repo.upsert({ id: 'ls-1', plugin: 'tmdb', service: 'tmdb', label: 'TMDB', media_type_id: 'video', category_id: 'movies' });
		repo.upsert({ id: 'ls-2', plugin: 'tmdb', service: 'tmdb', label: 'TMDB', media_type_id: 'video', category_id: 'tv' });
		repo.upsert({ id: 'ls-3', plugin: 'mb', service: 'musicbrainz', label: 'MusicBrainz', media_type_id: 'audio', category_id: null });

		const deleted = repo.deleteByPlugin('tmdb');
		expect(deleted).toBe(2);
		expect(repo.getAll()).toHaveLength(1);
		expect(repo.getAll()[0].plugin).toBe('mb');
	});

	it('should return 0 when deleting non-existent plugin sources', () => {
		expect(repo.deleteByPlugin('nonexistent')).toBe(0);
	});

	it('should support null category_id for type-wide sources', () => {
		repo.upsert({ id: 'ls-1', plugin: 'tmdb', service: 'tmdb', label: 'TMDB', media_type_id: 'video', category_id: null });
		const all = repo.getAll();
		expect(all).toHaveLength(1);
		expect(all[0].category_id).toBeNull();
	});
});
