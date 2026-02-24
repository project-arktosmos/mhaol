import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import Database from 'better-sqlite3';
import { initializeSchema } from '../../src/schema.js';
import { SettingsRepository } from '../../src/repositories/settings.repository.js';

describe('SettingsRepository', () => {
	let db: InstanceType<typeof Database>;
	let repo: SettingsRepository;

	beforeEach(() => {
		db = new Database(':memory:');
		db.pragma('foreign_keys = ON');
		initializeSchema(db);
		repo = new SettingsRepository(db);
	});

	afterEach(() => {
		db?.close();
	});

	it('should return null for a non-existent key', () => {
		expect(repo.get('nonexistent')).toBeNull();
	});

	it('should set and get a value', () => {
		repo.set('theme', 'dark');
		expect(repo.get('theme')).toBe('dark');
	});

	it('should overwrite an existing value', () => {
		repo.set('theme', 'dark');
		repo.set('theme', 'light');
		expect(repo.get('theme')).toBe('light');
	});

	it('should return a full row with getRow', () => {
		repo.set('lang', 'en');
		const row = repo.getRow('lang');
		expect(row).not.toBeNull();
		expect(row!.key).toBe('lang');
		expect(row!.value).toBe('en');
		expect(row!.created_at).toBeDefined();
		expect(row!.updated_at).toBeDefined();
	});

	it('should return null from getRow for non-existent key', () => {
		expect(repo.getRow('missing')).toBeNull();
	});

	it('should return all settings with getAll', () => {
		repo.set('a', '1');
		repo.set('b', '2');
		repo.set('c', '3');
		const all = repo.getAll();
		expect(all).toHaveLength(3);
		expect(all.map((r) => r.key)).toEqual(['a', 'b', 'c']);
	});

	it('should delete an existing key and return true', () => {
		repo.set('temp', 'value');
		expect(repo.delete('temp')).toBe(true);
		expect(repo.get('temp')).toBeNull();
	});

	it('should return false when deleting a non-existent key', () => {
		expect(repo.delete('nonexistent')).toBe(false);
	});

	it('should check existence with exists', () => {
		expect(repo.exists('theme')).toBe(false);
		repo.set('theme', 'dark');
		expect(repo.exists('theme')).toBe(true);
	});

	it('should set many values atomically with setMany', () => {
		repo.setMany({ a: '1', b: '2', c: '3' });
		expect(repo.get('a')).toBe('1');
		expect(repo.get('b')).toBe('2');
		expect(repo.get('c')).toBe('3');
	});

	it('should overwrite values with setMany', () => {
		repo.set('a', 'old');
		repo.setMany({ a: 'new', b: '2' });
		expect(repo.get('a')).toBe('new');
		expect(repo.get('b')).toBe('2');
	});
});
