import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import Database from 'better-sqlite3';
import { AddonCacheRepository, ADDON_CACHE_SCHEMA } from '../src/addon-cache';

describe('AddonCacheRepository', () => {
	let db: InstanceType<typeof Database>;
	let cache: AddonCacheRepository;

	beforeEach(() => {
		db = new Database(':memory:');
		db.exec(ADDON_CACHE_SCHEMA);
		cache = new AddonCacheRepository(db);
	});

	afterEach(() => {
		db?.close();
	});

	describe('data cache', () => {
		it('returns null for non-existent entry', () => {
			expect(cache.get('tmdb', 'movie', '550')).toBeNull();
		});

		it('upserts and retrieves an entry', () => {
			const data = JSON.stringify({ id: 550, title: 'Fight Club' });
			cache.upsert('tmdb', 'movie', '550', data);
			const row = cache.get('tmdb', 'movie', '550');
			expect(row).not.toBeNull();
			expect(row!.service).toBe('tmdb');
			expect(row!.resource_type).toBe('movie');
			expect(row!.cache_key).toBe('550');
			expect(row!.data).toBe(data);
			expect(row!.fetched_at).toBeDefined();
		});

		it('updates data on upsert', () => {
			cache.upsert('tmdb', 'movie', '550', '{"v":1}');
			cache.upsert('tmdb', 'movie', '550', '{"v":2}');
			const row = cache.get('tmdb', 'movie', '550');
			expect(row!.data).toBe('{"v":2}');
		});

		it('isolates by service', () => {
			cache.upsert('tmdb', 'movie', '1', '{"source":"tmdb"}');
			cache.upsert('musicbrainz', 'movie', '1', '{"source":"mb"}');
			expect(cache.get('tmdb', 'movie', '1')!.data).toBe('{"source":"tmdb"}');
			expect(cache.get('musicbrainz', 'movie', '1')!.data).toBe('{"source":"mb"}');
		});

		it('isolates by resource type', () => {
			cache.upsert('tmdb', 'movie', '1', '{"type":"movie"}');
			cache.upsert('tmdb', 'tv', '1', '{"type":"tv"}');
			expect(cache.get('tmdb', 'movie', '1')!.data).toBe('{"type":"movie"}');
			expect(cache.get('tmdb', 'tv', '1')!.data).toBe('{"type":"tv"}');
		});

		it('deletes an entry', () => {
			cache.upsert('tmdb', 'movie', '550', '{}');
			expect(cache.delete('tmdb', 'movie', '550')).toBe(true);
			expect(cache.get('tmdb', 'movie', '550')).toBeNull();
		});

		it('returns false when deleting non-existent entry', () => {
			expect(cache.delete('tmdb', 'movie', '999')).toBe(false);
		});

		it('deletes all entries for a service', () => {
			cache.upsert('tmdb', 'movie', '1', '{}');
			cache.upsert('tmdb', 'tv', '2', '{}');
			cache.upsert('musicbrainz', 'release', '3', '{}');
			expect(cache.deleteByService('tmdb')).toBe(2);
			expect(cache.get('tmdb', 'movie', '1')).toBeNull();
			expect(cache.get('musicbrainz', 'release', '3')).not.toBeNull();
		});

		it('deletes entries by service and type', () => {
			cache.upsert('tmdb', 'movie', '1', '{}');
			cache.upsert('tmdb', 'movie', '2', '{}');
			cache.upsert('tmdb', 'tv', '1', '{}');
			expect(cache.deleteByServiceType('tmdb', 'movie')).toBe(2);
			expect(cache.get('tmdb', 'movie', '1')).toBeNull();
			expect(cache.get('tmdb', 'tv', '1')).not.toBeNull();
		});
	});

	describe('image cache', () => {
		it('returns null for non-existent image', () => {
			expect(cache.getImage('tmdb', 'w342/abc.jpg')).toBeNull();
		});

		it('upserts and retrieves an image entry', () => {
			cache.upsertImage(
				'tmdb',
				'w342/abc.jpg',
				'https://image.tmdb.org/t/p/w342/abc.jpg',
				'/data/tmdb-images/w342/abc.jpg',
				'image/jpeg'
			);
			const row = cache.getImage('tmdb', 'w342/abc.jpg');
			expect(row).not.toBeNull();
			expect(row!.service).toBe('tmdb');
			expect(row!.url).toBe('https://image.tmdb.org/t/p/w342/abc.jpg');
			expect(row!.local_path).toBe('/data/tmdb-images/w342/abc.jpg');
			expect(row!.mime_type).toBe('image/jpeg');
		});

		it('updates on upsert', () => {
			cache.upsertImage('tmdb', 'img1', 'https://old.url', '/old/path');
			cache.upsertImage('tmdb', 'img1', 'https://new.url', '/new/path', 'image/webp');
			const row = cache.getImage('tmdb', 'img1');
			expect(row!.url).toBe('https://new.url');
			expect(row!.local_path).toBe('/new/path');
			expect(row!.mime_type).toBe('image/webp');
		});

		it('deletes an image entry', () => {
			cache.upsertImage('tmdb', 'img1', 'url', '/path');
			expect(cache.deleteImage('tmdb', 'img1')).toBe(true);
			expect(cache.getImage('tmdb', 'img1')).toBeNull();
		});

		it('deletes all images for a service', () => {
			cache.upsertImage('tmdb', 'img1', 'u1', '/p1');
			cache.upsertImage('tmdb', 'img2', 'u2', '/p2');
			cache.upsertImage('ra', 'img3', 'u3', '/p3');
			expect(cache.deleteImagesByService('tmdb')).toBe(2);
			expect(cache.getImage('ra', 'img3')).not.toBeNull();
		});
	});

	describe('freshness', () => {
		it('returns true for a recent timestamp', () => {
			const now = new Date().toISOString().replace('T', ' ').replace('Z', '').split('.')[0];
			expect(cache.isFresh(now)).toBe(true);
		});

		it('returns false for an old timestamp', () => {
			const old = new Date(Date.now() - 8 * 24 * 60 * 60 * 1000)
				.toISOString()
				.replace('T', ' ')
				.replace('Z', '')
				.split('.')[0];
			expect(cache.isFresh(old)).toBe(false);
		});

		it('returns true for timestamp within stale window', () => {
			const sixDaysAgo = new Date(Date.now() - 6 * 24 * 60 * 60 * 1000)
				.toISOString()
				.replace('T', ' ')
				.replace('Z', '')
				.split('.')[0];
			expect(cache.isFresh(sixDaysAgo)).toBe(true);
		});

		it('supports custom stale days override', () => {
			const twoDaysAgo = new Date(Date.now() - 2 * 24 * 60 * 60 * 1000)
				.toISOString()
				.replace('T', ' ')
				.replace('Z', '')
				.split('.')[0];
			expect(cache.isFresh(twoDaysAgo, 1)).toBe(false);
			expect(cache.isFresh(twoDaysAgo, 3)).toBe(true);
		});

		it('getFresh returns null for stale entry', () => {
			const staleCache = new AddonCacheRepository(db, 0);
			staleCache.upsert('tmdb', 'movie', '1', '{}');
			expect(staleCache.getFresh('tmdb', 'movie', '1')).toBeNull();
		});

		it('getFresh returns entry when fresh', () => {
			cache.upsert('tmdb', 'movie', '1', '{}');
			const row = cache.getFresh('tmdb', 'movie', '1');
			expect(row).not.toBeNull();
		});
	});
});
