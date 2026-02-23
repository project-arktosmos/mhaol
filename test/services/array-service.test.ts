import { describe, it, expect, beforeEach, vi } from 'vitest';
import { get } from 'svelte/store';
import { ArrayServiceClass } from '../../src/services/classes/array-service.class';
import { createMockItem, getLocalStorageItem } from '../utils/test-helpers';

interface TestItem {
	id: number;
	name: string;
}

describe('ArrayServiceClass', () => {
	let service: ArrayServiceClass<TestItem>;
	const initialItems: TestItem[] = [
		{ id: 1, name: 'Item 1' },
		{ id: 2, name: 'Item 2' }
	];

	beforeEach(() => {
		service = new ArrayServiceClass<TestItem>('test', initialItems);
	});

	describe('Initialization', () => {
		it('should initialize with correct id', () => {
			expect(service.id).toBe('array-service:test');
		});

		it('should initialize store with initial values', () => {
			const storeValue = get(service.store);
			expect(storeValue).toEqual(initialItems);
		});

		it('should set data property to initial values', () => {
			expect(service.data).toEqual(initialItems);
		});

		it('should persist initial data to localStorage', () => {
			const stored = getLocalStorageItem<TestItem[]>('array-service:test');
			expect(stored).toEqual(initialItems);
		});

		it('should log initialization message', () => {
			expect(console.info).toHaveBeenCalledWith('Service array-service:test initialized');
		});
	});

	describe('add()', () => {
		it('should add a new item to the store', () => {
			const newItem = createMockItem<TestItem>(3, { name: 'Item 3' });
			const result = service.add(newItem);

			expect(result).toEqual(newItem);
			expect(get(service.store)).toContainEqual(newItem);
			expect(get(service.store)).toHaveLength(3);
		});

		it('should update data property when adding item', () => {
			const newItem = createMockItem<TestItem>(3, { name: 'Item 3' });
			service.add(newItem);

			expect(service.data).toContainEqual(newItem);
		});

		it('should persist added item to localStorage', () => {
			const newItem = createMockItem<TestItem>(3, { name: 'Item 3' });
			service.add(newItem);

			const stored = getLocalStorageItem<TestItem[]>('array-service:test');
			expect(stored).toContainEqual(newItem);
		});

		it('should throw error when adding duplicate item by id', () => {
			const duplicateItem = { id: 1, name: 'Duplicate' };

			expect(() => service.add(duplicateItem)).toThrow();
		});

		it('should return the added item', () => {
			const newItem = createMockItem<TestItem>(3, { name: 'Item 3' });
			const result = service.add(newItem);

			expect(result).toBe(newItem);
		});
	});

	describe('remove()', () => {
		it('should remove an existing item from the store', () => {
			const itemToRemove = initialItems[0];
			service.remove(itemToRemove);

			expect(get(service.store)).not.toContainEqual(itemToRemove);
			expect(get(service.store)).toHaveLength(1);
		});

		it('should update data property when removing item', () => {
			const itemToRemove = initialItems[0];
			service.remove(itemToRemove);

			expect(service.data).not.toContainEqual(itemToRemove);
		});

		it('should persist removal to localStorage', () => {
			const itemToRemove = initialItems[0];
			service.remove(itemToRemove);

			const stored = getLocalStorageItem<TestItem[]>('array-service:test');
			expect(stored).not.toContainEqual(itemToRemove);
		});

		it('should throw error when removing non-existent item', () => {
			const nonExistentItem = createMockItem<TestItem>(999, { name: 'Non-existent' });

			expect(() => service.remove(nonExistentItem)).toThrow();
		});
	});

	describe('update()', () => {
		it('should update an existing item in the store', () => {
			const updatedItem = { id: 1, name: 'Updated Item 1' };
			const result = service.update(updatedItem);

			expect(result).toEqual(updatedItem);
			expect(get(service.store)[0]).toEqual(updatedItem);
		});

		it('should update data property when updating item', () => {
			const updatedItem = { id: 1, name: 'Updated Item 1' };
			service.update(updatedItem);

			expect(service.data[0]).toEqual(updatedItem);
		});

		it('should persist update to localStorage', () => {
			const updatedItem = { id: 1, name: 'Updated Item 1' };
			service.update(updatedItem);

			const stored = getLocalStorageItem<TestItem[]>('array-service:test');
			expect(stored![0]).toEqual(updatedItem);
		});

		it('should throw error when updating non-existent item', () => {
			const nonExistentItem = createMockItem<TestItem>(999, { name: 'Non-existent' });

			expect(() => service.update(nonExistentItem)).toThrow();
		});

		it('should return the updated item', () => {
			const updatedItem = { id: 1, name: 'Updated Item 1' };
			const result = service.update(updatedItem);

			expect(result).toBe(updatedItem);
		});
	});

	describe('exists()', () => {
		it('should return item if it exists', () => {
			const result = service.exists(1);

			expect(result).toEqual(initialItems[0]);
		});

		it('should return null if item does not exist', () => {
			const result = service.exists(999);

			expect(result).toBeNull();
		});

		it('should work with string IDs', () => {
			const serviceWithStringIds = new ArrayServiceClass<{ id: string; value: number }>(
				'string-test',
				[{ id: 'abc', value: 123 }]
			);

			const result = serviceWithStringIds.exists('abc');
			expect(result).toEqual({ id: 'abc', value: 123 });
		});
	});

	describe('all()', () => {
		it('should return all items', () => {
			const result = service.all();

			expect(result).toEqual(initialItems);
		});

		it('should return empty array when no items exist', () => {
			const emptyService = new ArrayServiceClass<TestItem>('empty', []);
			const result = emptyService.all();

			expect(result).toEqual([]);
		});

		it('should return current state after modifications', () => {
			const newItem = createMockItem<TestItem>(3, { name: 'Item 3' });
			service.add(newItem);

			const result = service.all();
			expect(result).toHaveLength(3);
			expect(result).toContainEqual(newItem);
		});
	});

	describe('find()', () => {
		it('should find item matching predicate', () => {
			const result = service.find((item) => item.id === 1);

			expect(result).toEqual(expect.objectContaining({ id: 1 }));
		});

		it('should return null if no item matches predicate', () => {
			const result = service.find((item) => item.name === 'Non-existent');

			expect(result).toBeNull();
		});

		it('should return first matching item', () => {
			service.add({ id: 3, name: 'TestName' });
			service.add({ id: 4, name: 'TestName' }); // Duplicate name, different id
			const result = service.find((item) => item.name === 'TestName');

			expect(result?.id).toBe(3);
		});
	});

	describe('filter()', () => {
		it('should filter items matching predicate', () => {
			const result = service.filter((item) => item.id > 1);

			expect(result).toHaveLength(1);
			expect(result[0]).toEqual(initialItems[1]);
		});

		it('should return empty array if no items match', () => {
			const result = service.filter((item) => item.id > 999);

			expect(result).toEqual([]);
		});

		it('should return all items if all match', () => {
			const result = service.filter((item) => item.id > 0);

			expect(result).toEqual(initialItems);
		});

		it('should work with complex predicates', () => {
			service.add({ id: 3, name: 'Special Item' });
			const result = service.filter((item) => item.name.includes('Item') && item.id < 3);

			expect(result).toHaveLength(2);
		});
	});

	describe('log()', () => {
		it('should log messages to console.info', () => {
			vi.clearAllMocks();
			service.log('test message', { data: 'value' });

			expect(console.info).toHaveBeenCalledWith('test message', { data: 'value' });
		});
	});

	describe('error()', () => {
		it('should throw an error with service id prefix', () => {
			expect(() => service.error('test error')).toThrow(
				'Service array-service:test error: test error'
			);
		});
	});

	describe('Store subscription and data sync', () => {
		it('should keep data property in sync with store updates', () => {
			const newItem = createMockItem<TestItem>(3, { name: 'Item 3' });
			service.add(newItem);

			expect(service.data).toEqual(get(service.store));
		});

		it('should log updates when store changes', () => {
			vi.clearAllMocks();
			const newItem = createMockItem<TestItem>(3, { name: 'Item 3' });
			service.add(newItem);

			expect(console.info).toHaveBeenCalledWith(
				'Service array-service:test updated',
				expect.arrayContaining([newItem])
			);
		});
	});
});
