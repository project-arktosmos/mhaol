import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import Database from 'better-sqlite3';
import { initializeSchema } from '../../src/schema.js';
import { MetadataRepository } from '../../src/repositories/metadata.repository.js';

describe('MetadataRepository', () => {
	let db: InstanceType<typeof Database>;
	let repo: MetadataRepository;

	beforeEach(() => {
		db = new Database(':memory:');
		db.pragma('foreign_keys = ON');
		initializeSchema(db);
		repo = new MetadataRepository(db);
	});

	afterEach(() => {
		db?.close();
	});

	it('should return null for a non-existent key', () => {
		expect(repo.get('nonexistent')).toBeNull();
	});

	it('should set and get a string value', () => {
		repo.set('name', 'test-app');
		const row = repo.get('name');
		expect(row).not.toBeNull();
		expect(row!.value).toBe('test-app');
		expect(row!.type).toBe('string');
	});

	it('should set and get a number value', () => {
		repo.set('count', 42);
		const row = repo.get('count');
		expect(row!.value).toBe('42');
		expect(row!.type).toBe('number');
	});

	it('should set and get a boolean value', () => {
		repo.set('enabled', true);
		const row = repo.get('enabled');
		expect(row!.value).toBe('true');
		expect(row!.type).toBe('boolean');
	});

	it('should set and get a json value', () => {
		repo.set('config', { foo: 'bar', num: 1 });
		const row = repo.get('config');
		expect(row!.value).toBe('{"foo":"bar","num":1}');
		expect(row!.type).toBe('json');
	});

	it('should parse string values with getValue', () => {
		repo.set('name', 'test');
		expect(repo.getValue('name')).toBe('test');
	});

	it('should parse number values with getValue', () => {
		repo.set('count', 42);
		expect(repo.getValue<number>('count')).toBe(42);
	});

	it('should parse boolean values with getValue', () => {
		repo.set('enabled', true);
		expect(repo.getValue<boolean>('enabled')).toBe(true);

		repo.set('disabled', false);
		expect(repo.getValue<boolean>('disabled')).toBe(false);
	});

	it('should parse json values with getValue', () => {
		const data = { items: [1, 2, 3], nested: { a: 'b' } };
		repo.set('data', data);
		expect(repo.getValue('data')).toEqual(data);
	});

	it('should return null from getValue for non-existent key', () => {
		expect(repo.getValue('missing')).toBeNull();
	});

	it('should overwrite existing values', () => {
		repo.set('key', 'string-value');
		repo.set('key', 123);
		const row = repo.get('key');
		expect(row!.value).toBe('123');
		expect(row!.type).toBe('number');
	});

	it('should return all metadata with getAll', () => {
		// Seed data already has db_version and created_at
		const all = repo.getAll();
		expect(all.length).toBeGreaterThanOrEqual(2);
		expect(all.map((r) => r.key)).toContain('db_version');
		expect(all.map((r) => r.key)).toContain('created_at');
	});

	it('should filter by type with getByType', () => {
		repo.set('str1', 'a');
		repo.set('str2', 'b');
		repo.set('num1', 1);

		const strings = repo.getByType('string');
		expect(strings.every((r) => r.type === 'string')).toBe(true);
		expect(strings.map((r) => r.key)).toContain('str1');
		expect(strings.map((r) => r.key)).toContain('str2');

		const numbers = repo.getByType('number');
		expect(numbers.every((r) => r.type === 'number')).toBe(true);
		expect(numbers.map((r) => r.key)).toContain('num1');
	});

	it('should delete an existing key and return true', () => {
		repo.set('temp', 'value');
		expect(repo.delete('temp')).toBe(true);
		expect(repo.get('temp')).toBeNull();
	});

	it('should return false when deleting a non-existent key', () => {
		expect(repo.delete('nonexistent')).toBe(false);
	});
});
