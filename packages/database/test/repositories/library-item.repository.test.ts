import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import Database from 'better-sqlite3';
import { initializeSchema } from '../../src/schema.js';
import { LibraryRepository } from '../../src/repositories/library.repository.js';
import { LibraryItemRepository } from '../../src/repositories/library-item.repository.js';

function makeItem(overrides: Partial<{
	id: string;
	library_id: string;
	path: string;
	extension: string;
	media_type: string;
	category_id: string | null;
}> = {}) {
	return {
		id: 'item-1',
		library_id: 'lib-1',
		path: '/media/movies/film.mp4',
		extension: 'mp4',
		media_type: 'video',
		category_id: null,
		...overrides
	};
}

describe('LibraryItemRepository', () => {
	let db: InstanceType<typeof Database>;
	let libraryRepo: LibraryRepository;
	let repo: LibraryItemRepository;

	beforeEach(() => {
		db = new Database(':memory:');
		db.pragma('foreign_keys = ON');
		initializeSchema(db);
		libraryRepo = new LibraryRepository(db);
		repo = new LibraryItemRepository(db);

		// Seed a library for items to reference
		libraryRepo.insert({
			id: 'lib-1',
			name: 'Movies',
			path: '/media/movies',
			media_types: '["video"]',
			date_added: 1000
		});
	});

	afterEach(() => {
		db?.close();
	});

	it('should return null for a non-existent id', () => {
		expect(repo.get('nonexistent')).toBeNull();
	});

	it('should return empty array when no items exist for a library', () => {
		expect(repo.getByLibrary('lib-1')).toEqual([]);
	});

	it('should insert and retrieve an item', () => {
		repo.insert(makeItem());
		const item = repo.get('item-1');
		expect(item).not.toBeNull();
		expect(item!.library_id).toBe('lib-1');
		expect(item!.path).toBe('/media/movies/film.mp4');
		expect(item!.extension).toBe('mp4');
		expect(item!.media_type).toBe('video');
		expect(item!.category_id).toBeNull();
		expect(item!.created_at).toBeDefined();
		expect(item!.updated_at).toBeDefined();
	});

	it('should get items by library ordered by path', () => {
		repo.insert(makeItem({ id: 'item-b', path: '/media/movies/b.mp4' }));
		repo.insert(makeItem({ id: 'item-a', path: '/media/movies/a.mp4' }));
		const items = repo.getByLibrary('lib-1');
		expect(items).toHaveLength(2);
		expect(items[0].path).toBe('/media/movies/a.mp4');
		expect(items[1].path).toBe('/media/movies/b.mp4');
	});

	it('should not return items from other libraries', () => {
		libraryRepo.insert({
			id: 'lib-2',
			name: 'Music',
			path: '/media/music',
			media_types: '["audio"]',
			date_added: 2000
		});
		repo.insert(makeItem({ id: 'item-1', path: '/media/movies/film.mp4' }));
		repo.insert(makeItem({ id: 'item-2', library_id: 'lib-2', path: '/media/music/song.mp3', extension: 'mp3', media_type: 'audio' }));
		expect(repo.getByLibrary('lib-1')).toHaveLength(1);
		expect(repo.getByLibrary('lib-2')).toHaveLength(1);
	});

	it('should bulk insert with insertMany', () => {
		repo.insertMany([
			makeItem({ id: 'item-1', path: '/media/movies/a.mp4' }),
			makeItem({ id: 'item-2', path: '/media/movies/b.mkv', extension: 'mkv' }),
			makeItem({ id: 'item-3', path: '/media/movies/c.avi', extension: 'avi' })
		]);
		expect(repo.getByLibrary('lib-1')).toHaveLength(3);
	});

	it('should delete an item and return true', () => {
		repo.insert(makeItem());
		expect(repo.delete('item-1')).toBe(true);
		expect(repo.get('item-1')).toBeNull();
	});

	it('should return false when deleting a non-existent item', () => {
		expect(repo.delete('nonexistent')).toBe(false);
	});

	it('should delete all items for a library', () => {
		repo.insertMany([
			makeItem({ id: 'item-1', path: '/media/movies/a.mp4' }),
			makeItem({ id: 'item-2', path: '/media/movies/b.mp4' })
		]);
		const deleted = repo.deleteByLibrary('lib-1');
		expect(deleted).toBe(2);
		expect(repo.getByLibrary('lib-1')).toEqual([]);
	});

	it('should check if a path exists', () => {
		expect(repo.existsByPath('/media/movies/film.mp4')).toBeNull();
		repo.insert(makeItem());
		expect(repo.existsByPath('/media/movies/film.mp4')).toBe('item-1');
	});

	it('should enforce unique path constraint', () => {
		repo.insert(makeItem());
		expect(() => repo.insert(makeItem({ id: 'item-2' }))).toThrow();
	});

	it('should cascade delete items when library is deleted', () => {
		repo.insertMany([
			makeItem({ id: 'item-1', path: '/media/movies/a.mp4' }),
			makeItem({ id: 'item-2', path: '/media/movies/b.mp4' })
		]);
		libraryRepo.delete('lib-1');
		expect(repo.getByLibrary('lib-1')).toEqual([]);
		expect(repo.get('item-1')).toBeNull();
		expect(repo.get('item-2')).toBeNull();
	});

	it('should reject invalid media_type values', () => {
		expect(() => repo.insert(makeItem({ media_type: 'document', path: '/media/movies/file.txt', extension: 'txt' }))).toThrow();
	});

	// Category tests

	it('should insert an item with a category', () => {
		repo.insert(makeItem({ category_id: 'movies' }));
		const item = repo.get('item-1');
		expect(item!.category_id).toBe('movies');
	});

	it('should update category on an existing item', () => {
		repo.insert(makeItem());
		expect(repo.get('item-1')!.category_id).toBeNull();

		repo.updateCategory('item-1', 'tv');
		expect(repo.get('item-1')!.category_id).toBe('tv');
	});

	it('should clear category on an existing item', () => {
		repo.insert(makeItem({ category_id: 'movies' }));
		expect(repo.get('item-1')!.category_id).toBe('movies');

		repo.clearCategory('item-1');
		expect(repo.get('item-1')!.category_id).toBeNull();
	});

	it('should get items by category', () => {
		repo.insertMany([
			makeItem({ id: 'item-1', path: '/media/movies/a.mp4', category_id: 'movies' }),
			makeItem({ id: 'item-2', path: '/media/movies/b.mp4', category_id: 'movies' }),
			makeItem({ id: 'item-3', path: '/media/movies/c.mp4', category_id: 'tv' })
		]);
		const results = repo.getByCategory('movies');
		expect(results).toHaveLength(2);
		expect(results.map((r) => r.id).sort()).toEqual(['item-1', 'item-2']);
	});

	it('should return empty array for unmatched category', () => {
		repo.insert(makeItem({ category_id: 'movies' }));
		expect(repo.getByCategory('tv')).toEqual([]);
	});

	it('should update media type on an existing item', () => {
		repo.insert(makeItem());
		expect(repo.get('item-1')!.media_type).toBe('video');

		repo.updateMediaType('item-1', 'audio');
		expect(repo.get('item-1')!.media_type).toBe('audio');
	});
});
