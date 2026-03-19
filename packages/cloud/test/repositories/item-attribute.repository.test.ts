import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import Database from 'better-sqlite3';
import { initializeCloudSchema } from '../../src/schema.js';
import { CloudLibraryRepository } from '../../src/repositories/library.repository.js';
import { CloudItemRepository } from '../../src/repositories/item.repository.js';
import { ItemAttributeRepository } from '../../src/repositories/item-attribute.repository.js';

describe('ItemAttributeRepository', () => {
	let db: InstanceType<typeof Database>;
	let libraryRepo: CloudLibraryRepository;
	let itemRepo: CloudItemRepository;
	let attrRepo: ItemAttributeRepository;

	beforeEach(() => {
		db = new Database(':memory:');
		db.pragma('foreign_keys = ON');
		initializeCloudSchema(db);
		libraryRepo = new CloudLibraryRepository(db);
		itemRepo = new CloudItemRepository(db);
		attrRepo = new ItemAttributeRepository(db);

		libraryRepo.insert({ id: 'lib-1', name: 'Test', path: '/test', kind: 'filesystem' });
		itemRepo.insert({
			id: 'item-1',
			library_id: 'lib-1',
			path: '/test/song.mp3',
			filename: 'song.mp3',
			extension: 'mp3',
			size_bytes: 5000000,
			mime_type: 'audio/mpeg',
			checksum: null
		});
		itemRepo.insert({
			id: 'item-2',
			library_id: 'lib-1',
			path: '/test/video.mp4',
			filename: 'video.mp4',
			extension: 'mp4',
			size_bytes: 50000000,
			mime_type: 'video/mp4',
			checksum: null
		});
	});

	afterEach(() => {
		db?.close();
	});

	it('should return empty array for item with no attributes', () => {
		expect(attrRepo.getByItem('item-1')).toEqual([]);
	});

	it('should set and retrieve an attribute', () => {
		attrRepo.set({
			id: 'attr-1',
			item_id: 'item-1',
			key: 'artist',
			value: 'Test Artist',
			attribute_type_id: 'string',
			source: 'user',
			confidence: null
		});

		const attrs = attrRepo.getByItem('item-1');
		expect(attrs).toHaveLength(1);
		expect(attrs[0].key).toBe('artist');
		expect(attrs[0].value).toBe('Test Artist');
		expect(attrs[0].attribute_type_id).toBe('string');
		expect(attrs[0].source).toBe('user');
	});

	it('should upsert on (item_id, key, source) conflict', () => {
		attrRepo.set({
			id: 'attr-1',
			item_id: 'item-1',
			key: 'title',
			value: 'Old Title',
			attribute_type_id: 'string',
			source: 'system',
			confidence: null
		});
		attrRepo.set({
			id: 'attr-2',
			item_id: 'item-1',
			key: 'title',
			value: 'New Title',
			attribute_type_id: 'string',
			source: 'system',
			confidence: null
		});

		const attrs = attrRepo.getByItemAndKey('item-1', 'title');
		expect(attrs).toHaveLength(1);
		expect(attrs[0].value).toBe('New Title');
	});

	it('should allow same key from different sources', () => {
		attrRepo.set({
			id: 'attr-1',
			item_id: 'item-1',
			key: 'title',
			value: 'Filename Title',
			attribute_type_id: 'string',
			source: 'system',
			confidence: null
		});
		attrRepo.set({
			id: 'attr-2',
			item_id: 'item-1',
			key: 'title',
			value: 'MusicBrainz Title',
			attribute_type_id: 'string',
			source: 'musicbrainz',
			confidence: 0.95
		});

		const attrs = attrRepo.getByItemAndKey('item-1', 'title');
		expect(attrs).toHaveLength(2);
	});

	it('should set many attributes in a transaction', () => {
		attrRepo.setMany([
			{ id: 'a1', item_id: 'item-1', key: 'artist', value: 'Artist', attribute_type_id: 'string', source: 'user', confidence: null },
			{ id: 'a2', item_id: 'item-1', key: 'album', value: 'Album', attribute_type_id: 'string', source: 'user', confidence: null },
			{ id: 'a3', item_id: 'item-1', key: 'duration', value: '234', attribute_type_id: 'duration', source: 'system', confidence: null }
		]);

		expect(attrRepo.getByItem('item-1')).toHaveLength(3);
	});

	it('should get attributes by key across all items', () => {
		attrRepo.set({ id: 'a1', item_id: 'item-1', key: 'genre', value: 'Rock', attribute_type_id: 'string', source: 'user', confidence: null });
		attrRepo.set({ id: 'a2', item_id: 'item-2', key: 'genre', value: 'Pop', attribute_type_id: 'string', source: 'user', confidence: null });

		const genres = attrRepo.getByKey('genre');
		expect(genres).toHaveLength(2);
	});

	it('should get attributes by key and value', () => {
		attrRepo.set({ id: 'a1', item_id: 'item-1', key: 'genre', value: 'Rock', attribute_type_id: 'string', source: 'user', confidence: null });
		attrRepo.set({ id: 'a2', item_id: 'item-2', key: 'genre', value: 'Rock', attribute_type_id: 'string', source: 'user', confidence: null });

		const rockItems = attrRepo.getByKeyAndValue('genre', 'Rock');
		expect(rockItems).toHaveLength(2);
	});

	it('should search by value pattern', () => {
		attrRepo.set({ id: 'a1', item_id: 'item-1', key: 'artist', value: 'The Beatles', attribute_type_id: 'string', source: 'user', confidence: null });
		attrRepo.set({ id: 'a2', item_id: 'item-2', key: 'artist', value: 'Beat Happening', attribute_type_id: 'string', source: 'user', confidence: null });

		const results = attrRepo.search('artist', 'Beat');
		expect(results).toHaveLength(2);
	});

	it('should delete by item', () => {
		attrRepo.set({ id: 'a1', item_id: 'item-1', key: 'k1', value: 'v1', attribute_type_id: 'string', source: 'system', confidence: null });
		attrRepo.set({ id: 'a2', item_id: 'item-1', key: 'k2', value: 'v2', attribute_type_id: 'string', source: 'system', confidence: null });

		const deleted = attrRepo.deleteByItem('item-1');
		expect(deleted).toBe(2);
		expect(attrRepo.getByItem('item-1')).toEqual([]);
	});

	it('should delete by item and key', () => {
		attrRepo.set({ id: 'a1', item_id: 'item-1', key: 'artist', value: 'A', attribute_type_id: 'string', source: 'system', confidence: null });
		attrRepo.set({ id: 'a2', item_id: 'item-1', key: 'artist', value: 'B', attribute_type_id: 'string', source: 'musicbrainz', confidence: null });
		attrRepo.set({ id: 'a3', item_id: 'item-1', key: 'genre', value: 'Rock', attribute_type_id: 'string', source: 'system', confidence: null });

		const deleted = attrRepo.deleteByItemAndKey('item-1', 'artist');
		expect(deleted).toBe(2);
		expect(attrRepo.getByItem('item-1')).toHaveLength(1);
	});

	it('should return distinct keys', () => {
		attrRepo.set({ id: 'a1', item_id: 'item-1', key: 'artist', value: 'A', attribute_type_id: 'string', source: 'user', confidence: null });
		attrRepo.set({ id: 'a2', item_id: 'item-1', key: 'genre', value: 'Rock', attribute_type_id: 'string', source: 'user', confidence: null });
		attrRepo.set({ id: 'a3', item_id: 'item-2', key: 'genre', value: 'Pop', attribute_type_id: 'string', source: 'user', confidence: null });

		const keys = attrRepo.getDistinctKeys();
		expect(keys).toEqual(['artist', 'genre']);
	});

	it('should return distinct values for a key', () => {
		attrRepo.set({ id: 'a1', item_id: 'item-1', key: 'genre', value: 'Rock', attribute_type_id: 'string', source: 'user', confidence: null });
		attrRepo.set({ id: 'a2', item_id: 'item-2', key: 'genre', value: 'Pop', attribute_type_id: 'string', source: 'user', confidence: null });

		const values = attrRepo.getDistinctValues('genre');
		expect(values).toEqual(['Pop', 'Rock']);
	});

	it('should cascade delete attributes when item is deleted', () => {
		attrRepo.set({ id: 'a1', item_id: 'item-1', key: 'title', value: 'Song', attribute_type_id: 'string', source: 'system', confidence: null });
		itemRepo.delete('item-1');
		expect(attrRepo.getByItem('item-1')).toEqual([]);
	});
});
