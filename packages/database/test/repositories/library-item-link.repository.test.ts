import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import Database from 'better-sqlite3';
import { initializeSchema } from '../../src/schema.js';
import { LibraryRepository } from '../../src/repositories/library.repository.js';
import { LibraryItemRepository } from '../../src/repositories/library-item.repository.js';
import { LibraryItemLinkRepository } from '../../src/repositories/library-item-link.repository.js';

describe('LibraryItemLinkRepository', () => {
	let db: InstanceType<typeof Database>;
	let linkRepo: LibraryItemLinkRepository;

	beforeEach(() => {
		db = new Database(':memory:');
		db.pragma('foreign_keys = ON');
		initializeSchema(db);

		const libraryRepo = new LibraryRepository(db);
		const itemRepo = new LibraryItemRepository(db);
		linkRepo = new LibraryItemLinkRepository(db);

		libraryRepo.insert({
			id: 'lib-1',
			name: 'Movies',
			path: '/media/movies',
			media_types: '["video"]',
			date_added: 1000
		});
		itemRepo.insert({ id: 'item-1', library_id: 'lib-1', path: '/media/movies/a.mp4', extension: 'mp4', media_type: 'video', category_id: null });
		itemRepo.insert({ id: 'item-2', library_id: 'lib-1', path: '/media/movies/b.mp4', extension: 'mp4', media_type: 'video', category_id: null });
	});

	afterEach(() => {
		db?.close();
	});

	it('should return empty array when no links exist', () => {
		expect(linkRepo.getByItem('item-1')).toEqual([]);
	});

	it('should upsert and retrieve a link', () => {
		linkRepo.upsert({ id: 'link-1', library_item_id: 'item-1', service: 'tmdb', service_id: '550', season_number: null, episode_number: null });
		const links = linkRepo.getByItem('item-1');
		expect(links).toHaveLength(1);
		expect(links[0].service).toBe('tmdb');
		expect(links[0].service_id).toBe('550');
		expect(links[0].season_number).toBeNull();
	});

	it('should upsert with season and episode numbers', () => {
		linkRepo.upsert({ id: 'link-1', library_item_id: 'item-1', service: 'tmdb', service_id: '1399', season_number: 2, episode_number: 5 });
		const link = linkRepo.getByItemAndService('item-1', 'tmdb');
		expect(link).not.toBeNull();
		expect(link!.service_id).toBe('1399');
		expect(link!.season_number).toBe(2);
		expect(link!.episode_number).toBe(5);
	});

	it('should update existing link on upsert conflict', () => {
		linkRepo.upsert({ id: 'link-1', library_item_id: 'item-1', service: 'tmdb', service_id: '550', season_number: null, episode_number: null });
		linkRepo.upsert({ id: 'link-2', library_item_id: 'item-1', service: 'tmdb', service_id: '999', season_number: 1, episode_number: 3 });
		const links = linkRepo.getByItem('item-1');
		expect(links).toHaveLength(1);
		expect(links[0].service_id).toBe('999');
		expect(links[0].season_number).toBe(1);
	});

	it('should get link by item and service', () => {
		linkRepo.upsert({ id: 'link-1', library_item_id: 'item-1', service: 'tmdb', service_id: '550', season_number: null, episode_number: null });
		linkRepo.upsert({ id: 'link-2', library_item_id: 'item-1', service: 'youtube', service_id: 'abc123', season_number: null, episode_number: null });

		const tmdb = linkRepo.getByItemAndService('item-1', 'tmdb');
		expect(tmdb!.service_id).toBe('550');

		const yt = linkRepo.getByItemAndService('item-1', 'youtube');
		expect(yt!.service_id).toBe('abc123');

		expect(linkRepo.getByItemAndService('item-1', 'musicbrainz')).toBeNull();
	});

	it('should find links by service and service id', () => {
		linkRepo.upsert({ id: 'link-1', library_item_id: 'item-1', service: 'tmdb', service_id: '550', season_number: null, episode_number: null });
		linkRepo.upsert({ id: 'link-2', library_item_id: 'item-2', service: 'tmdb', service_id: '550', season_number: null, episode_number: null });
		linkRepo.upsert({ id: 'link-3', library_item_id: 'item-2', service: 'tmdb', service_id: '999', season_number: null, episode_number: null });

		const results = linkRepo.getByServiceId('tmdb', '550');
		expect(results).toHaveLength(1);
		expect(results[0].library_item_id).toBe('item-1');
	});

	it('should delete a link by item and service', () => {
		linkRepo.upsert({ id: 'link-1', library_item_id: 'item-1', service: 'tmdb', service_id: '550', season_number: null, episode_number: null });
		expect(linkRepo.delete('item-1', 'tmdb')).toBe(true);
		expect(linkRepo.getByItem('item-1')).toEqual([]);
	});

	it('should return false when deleting non-existent link', () => {
		expect(linkRepo.delete('item-1', 'tmdb')).toBe(false);
	});

	it('should delete all links for an item', () => {
		linkRepo.upsert({ id: 'link-1', library_item_id: 'item-1', service: 'tmdb', service_id: '550', season_number: null, episode_number: null });
		linkRepo.upsert({ id: 'link-2', library_item_id: 'item-1', service: 'youtube', service_id: 'abc', season_number: null, episode_number: null });
		const deleted = linkRepo.deleteByItem('item-1');
		expect(deleted).toBe(2);
		expect(linkRepo.getByItem('item-1')).toEqual([]);
	});

	it('should cascade delete links when library item is deleted', () => {
		linkRepo.upsert({ id: 'link-1', library_item_id: 'item-1', service: 'tmdb', service_id: '550', season_number: null, episode_number: null });
		db.prepare('DELETE FROM library_items WHERE id = ?').run('item-1');
		expect(linkRepo.getByItem('item-1')).toEqual([]);
	});
});
