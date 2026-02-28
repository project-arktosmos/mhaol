import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import Database from 'better-sqlite3';
import { initializeSchema } from '../../src/schema.js';
import { LibraryRepository } from '../../src/repositories/library.repository.js';

describe('LibraryRepository', () => {
	let db: InstanceType<typeof Database>;
	let repo: LibraryRepository;

	beforeEach(() => {
		db = new Database(':memory:');
		db.pragma('foreign_keys = ON');
		initializeSchema(db);
		repo = new LibraryRepository(db);
	});

	afterEach(() => {
		db?.close();
	});

	it('should return empty array when no libraries exist', () => {
		expect(repo.getAll()).toEqual([]);
	});

	it('should return null for a non-existent id', () => {
		expect(repo.get('nonexistent')).toBeNull();
	});

	it('should insert and retrieve a library', () => {
		repo.insert({
			id: 'lib-1',
			name: 'Movies',
			path: '/media/movies',
			media_types: '["video"]',
			date_added: 1000
		});
		const lib = repo.get('lib-1');
		expect(lib).not.toBeNull();
		expect(lib!.name).toBe('Movies');
		expect(lib!.path).toBe('/media/movies');
		expect(JSON.parse(lib!.media_types)).toEqual(['video']);
		expect(lib!.date_added).toBe(1000);
		expect(lib!.created_at).toBeDefined();
		expect(lib!.updated_at).toBeDefined();
	});

	it('should return all libraries ordered by date_added DESC', () => {
		repo.insert({ id: 'a', name: 'A', path: '/a', media_types: '[]', date_added: 100 });
		repo.insert({ id: 'b', name: 'B', path: '/b', media_types: '[]', date_added: 200 });
		const all = repo.getAll();
		expect(all).toHaveLength(2);
		expect(all[0].id).toBe('b');
		expect(all[1].id).toBe('a');
	});

	it('should delete a library and return true', () => {
		repo.insert({ id: 'lib-1', name: 'X', path: '/x', media_types: '[]', date_added: 1 });
		expect(repo.delete('lib-1')).toBe(true);
		expect(repo.get('lib-1')).toBeNull();
	});

	it('should return false when deleting a non-existent library', () => {
		expect(repo.delete('nonexistent')).toBe(false);
	});

	it('should update a library', () => {
		repo.insert({
			id: 'lib-1',
			name: 'Old',
			path: '/old',
			media_types: '["video"]',
			date_added: 1
		});
		repo.update('lib-1', { name: 'New', path: '/new', media_types: '["audio","video"]' });
		const lib = repo.get('lib-1');
		expect(lib!.name).toBe('New');
		expect(lib!.path).toBe('/new');
		expect(JSON.parse(lib!.media_types)).toEqual(['audio', 'video']);
	});

	it('should store multiple media types as JSON', () => {
		const types = ['video', 'image', 'audio'];
		repo.insert({
			id: 'lib-1',
			name: 'All Media',
			path: '/media',
			media_types: JSON.stringify(types),
			date_added: 1
		});
		const lib = repo.get('lib-1');
		expect(JSON.parse(lib!.media_types)).toEqual(types);
	});
});
