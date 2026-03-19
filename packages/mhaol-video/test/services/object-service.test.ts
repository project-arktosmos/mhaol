import { describe, it, expect, beforeEach, vi } from 'vitest';
import { get } from 'svelte/store';
import { ObjectServiceClass } from '../../src/services/classes/object-service.class';
import { createMockItem, getLocalStorageItem } from '../utils/test-helpers';

interface TestObject {
	id: number;
	name: string;
	value: string;
}

describe('ObjectServiceClass', () => {
	let service: ObjectServiceClass<TestObject>;
	const initialObject: TestObject = { id: 1, name: 'Test Object', value: 'initial' };

	beforeEach(() => {
		service = new ObjectServiceClass<TestObject>(1, initialObject);
	});

	describe('Initialization', () => {
		it('should initialize with correct id', () => {
			expect(service.id).toBe('object-service:1');
		});

		it('should accept string or number as ID', () => {
			const serviceWithStringId = new ObjectServiceClass<TestObject>('test-id', initialObject);
			expect(serviceWithStringId.id).toBe('object-service:test-id');
		});

		it('should initialize store with initial value', () => {
			const storeValue = get(service.store);
			expect(storeValue).toEqual(initialObject);
		});

		it('should set data property to initial value', () => {
			expect(service.data).toEqual(initialObject);
		});

		it('should persist initial data to localStorage', () => {
			const stored = getLocalStorageItem<TestObject>('object-service:1');
			expect(stored).toEqual(initialObject);
		});

		it('should log initialization message', () => {
			expect(console.info).toHaveBeenCalledWith('Service object-service:1 initialized');
		});
	});

	describe('add()', () => {
		it('should add item when store is empty', () => {
			// Create a service with null/undefined initial value
			const emptyService = new ObjectServiceClass<TestObject | null>('empty', null);
			const newItem = createMockItem<TestObject>(2, { name: 'New Object', value: 'new' });

			emptyService.add(newItem);

			expect(get(emptyService.store)).toEqual(newItem);
		});

		it('should update data property when adding item', () => {
			const emptyService = new ObjectServiceClass<TestObject | null>('empty', null);
			const newItem = createMockItem<TestObject>(2, { name: 'New Object', value: 'new' });

			emptyService.add(newItem);

			expect(emptyService.data).toEqual(newItem);
		});

		it('should persist added item to localStorage', () => {
			const emptyService = new ObjectServiceClass<TestObject | null>('empty', null);
			const newItem = createMockItem<TestObject>(2, { name: 'New Object', value: 'new' });

			emptyService.add(newItem);

			const stored = getLocalStorageItem<TestObject>('object-service:empty');
			expect(stored).toEqual(newItem);
		});

		it('should throw error when item already exists', () => {
			const newItem = createMockItem<TestObject>(2, { name: 'Another Object', value: 'another' });

			expect(() => service.add(newItem)).toThrow();
		});

		it('should include service id in error message', () => {
			const newItem = createMockItem<TestObject>(2, { name: 'Another Object', value: 'another' });

			expect(() => service.add(newItem)).toThrow(/object-service:1/);
		});
	});

	describe('update()', () => {
		it('should update existing item', () => {
			const updatedItem: TestObject = { id: 1, name: 'Updated Object', value: 'updated' };
			service.update(updatedItem);

			expect(get(service.store)).toEqual(updatedItem);
		});

		it('should update data property', () => {
			const updatedItem: TestObject = { id: 1, name: 'Updated Object', value: 'updated' };
			service.update(updatedItem);

			expect(service.data).toEqual(updatedItem);
		});

		it('should persist update to localStorage', () => {
			const updatedItem: TestObject = { id: 1, name: 'Updated Object', value: 'updated' };
			service.update(updatedItem);

			const stored = getLocalStorageItem<TestObject>('object-service:1');
			expect(stored).toEqual(updatedItem);
		});

		it('should throw error when no item exists', () => {
			const emptyService = new ObjectServiceClass<TestObject | null>('empty', null);
			const newItem = createMockItem<TestObject>(2, { name: 'New Object', value: 'new' });

			expect(() => emptyService.update(newItem)).toThrow();
		});

		it('should include service id in error message', () => {
			const emptyService = new ObjectServiceClass<TestObject | null>('empty', null);
			const newItem = createMockItem<TestObject>(2, { name: 'New Object', value: 'new' });

			expect(() => emptyService.update(newItem)).toThrow(/object-service:empty/);
		});

		it('should allow updating with different id', () => {
			const updatedItem: TestObject = { id: 99, name: 'Different ID', value: 'different' };
			service.update(updatedItem);

			expect(get(service.store).id).toBe(99);
		});
	});

	describe('get()', () => {
		it('should return current object', () => {
			const result = service.get();

			expect(result).toEqual(initialObject);
		});

		it('should return updated object after update', () => {
			const updatedItem: TestObject = { id: 1, name: 'Updated', value: 'updated' };
			service.update(updatedItem);

			const result = service.get();
			expect(result).toEqual(updatedItem);
		});

		it('should return current state', () => {
			const result = service.get();
			expect(result).toBe(get(service.store));
		});
	});

	describe('set()', () => {
		it('should set a new object', () => {
			const newItem: TestObject = { id: 2, name: 'Set Object', value: 'set' };
			service.set(newItem);

			expect(get(service.store)).toEqual(newItem);
		});

		it('should update data property when setting', () => {
			const newItem: TestObject = { id: 2, name: 'Set Object', value: 'set' };
			service.set(newItem);

			expect(service.data).toEqual(newItem);
		});

		it('should persist set value to localStorage', () => {
			const newItem: TestObject = { id: 2, name: 'Set Object', value: 'set' };
			service.set(newItem);

			const stored = getLocalStorageItem<TestObject>('object-service:1');
			expect(stored).toEqual(newItem);
		});

		it('should replace existing object', () => {
			const newItem: TestObject = { id: 99, name: 'Completely New', value: 'new' };
			service.set(newItem);

			expect(get(service.store)).toEqual(newItem);
			expect(get(service.store)).not.toEqual(initialObject);
		});

		it('should work even when no object exists', () => {
			const emptyService = new ObjectServiceClass<TestObject | null>('empty', null);
			const newItem = createMockItem<TestObject>(1, { name: 'First Object', value: 'first' });

			emptyService.set(newItem);

			expect(get(emptyService.store)).toEqual(newItem);
		});
	});

	describe('log()', () => {
		it('should log messages to console.info', () => {
			vi.clearAllMocks();
			service.log('test message', { data: 'value' });

			expect(console.info).toHaveBeenCalledWith('test message', { data: 'value' });
		});

		it('should accept multiple arguments', () => {
			vi.clearAllMocks();
			service.log('message', 'arg1', 'arg2', { obj: 'value' });

			expect(console.info).toHaveBeenCalledWith('message', 'arg1', 'arg2', { obj: 'value' });
		});
	});

	describe('error()', () => {
		it('should throw an error with service id prefix', () => {
			expect(() => service.error('test error')).toThrow(
				'Service object-service:1 error: test error'
			);
		});

		it('should include error arguments in message', () => {
			expect(() => service.error('error message', 'additional info')).toThrow(
				/object-service:1 error/
			);
		});
	});

	describe('Store subscription and data sync', () => {
		it('should keep data property in sync with store updates', () => {
			const newItem: TestObject = { id: 2, name: 'Synced Object', value: 'synced' };
			service.set(newItem);

			expect(service.data).toEqual(get(service.store));
		});

		it('should log updates when store changes', () => {
			vi.clearAllMocks();
			const newItem: TestObject = { id: 2, name: 'Logged Object', value: 'logged' };
			service.set(newItem);

			expect(console.info).toHaveBeenCalledWith('Service object-service:1 updated', newItem);
		});

		it('should maintain sync through multiple operations', () => {
			const item1: TestObject = { id: 2, name: 'Item 1', value: 'first' };
			const item2: TestObject = { id: 3, name: 'Item 2', value: 'second' };

			service.set(item1);
			expect(service.data).toEqual(item1);

			service.update(item2);
			expect(service.data).toEqual(item2);
		});
	});

	describe('Edge cases', () => {
		it('should handle objects with additional properties', () => {
			interface ExtendedObject extends TestObject {
				extra: string;
			}

			const extendedService = new ObjectServiceClass<ExtendedObject>('extended', {
				id: 1,
				name: 'Extended',
				value: 'test',
				extra: 'data'
			});

			const newItem: ExtendedObject = { id: 2, name: 'New', value: 'new', extra: 'more' };
			extendedService.set(newItem);

			expect(extendedService.get()).toEqual(newItem);
		});

		it('should work with complex nested objects', () => {
			interface ComplexObject {
				id: number;
				nested: {
					deep: {
						value: string;
					};
				};
			}

			const complexService = new ObjectServiceClass<ComplexObject>('complex', {
				id: 1,
				nested: { deep: { value: 'initial' } }
			});

			const updated: ComplexObject = { id: 1, nested: { deep: { value: 'updated' } } };
			complexService.update(updated);

			expect(complexService.get().nested.deep.value).toBe('updated');
		});
	});
});
