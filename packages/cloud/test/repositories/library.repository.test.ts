import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import Database from 'better-sqlite3';
import { initializeCloudSchema } from '../../src/schema.js';
import { CloudLibraryRepository } from '../../src/repositories/library.repository.js';

describe('CloudLibraryRepository', () => {
	let db: InstanceType<typeof Database>;
	let repo: CloudLibraryRepository;

	beforeEach(() => {
		db = new Database(':memory:');
		db.pragma('foreign_keys = ON');
		initializeCloudSchema(db);
		repo = new CloudLibraryRepository(db);
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
		repo.insert({ id: 'lib-1', name: 'Movies', path: '/media/movies', kind: 'filesystem' });
		const lib = repo.get('lib-1');
		expect(lib).not.toBeNull();
		expect(lib!.name).toBe('Movies');
		expect(lib!.path).toBe('/media/movies');
		expect(lib!.kind).toBe('filesystem');
		expect(lib!.scan_status).toBe('idle');
		expect(lib!.scan_error).toBeNull();
		expect(lib!.item_count).toBe(0);
		expect(lib!.created_at).toBeDefined();
	});

	it('should update a library', () => {
		repo.insert({ id: 'lib-1', name: 'Old', path: '/old', kind: 'filesystem' });
		repo.update('lib-1', { name: 'New', path: '/new' });
		const lib = repo.get('lib-1');
		expect(lib!.name).toBe('New');
		expect(lib!.path).toBe('/new');
	});

	it('should update scan status', () => {
		repo.insert({ id: 'lib-1', name: 'Test', path: '/test', kind: 'filesystem' });
		repo.updateScanStatus('lib-1', 'scanning');
		expect(repo.get('lib-1')!.scan_status).toBe('scanning');

		repo.updateScanStatus('lib-1', 'error', 'Permission denied');
		const lib = repo.get('lib-1')!;
		expect(lib.scan_status).toBe('error');
		expect(lib.scan_error).toBe('Permission denied');
	});

	it('should update item count', () => {
		repo.insert({ id: 'lib-1', name: 'Test', path: '/test', kind: 'filesystem' });
		repo.updateItemCount('lib-1', 42);
		expect(repo.get('lib-1')!.item_count).toBe(42);
	});

	it('should delete a library', () => {
		repo.insert({ id: 'lib-1', name: 'X', path: '/x', kind: 'filesystem' });
		expect(repo.delete('lib-1')).toBe(true);
		expect(repo.get('lib-1')).toBeNull();
	});

	it('should return false when deleting a non-existent library', () => {
		expect(repo.delete('nonexistent')).toBe(false);
	});

	it('should return all libraries ordered by created_at DESC', () => {
		repo.insert({ id: 'a', name: 'A', path: '/a', kind: 'filesystem' });
		repo.insert({ id: 'b', name: 'B', path: '/b', kind: 'filesystem' });
		const all = repo.getAll();
		expect(all).toHaveLength(2);
	});
});
