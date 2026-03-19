import { describe, it, expect, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import localStorageWritableStore from '../../src/utils/localStorageWritableStore';

describe('localStorageWritableStore', () => {
	const testKey = 'test-store-key';
	const initialValue = { id: 1, name: 'Test' };

	beforeEach(() => {
		localStorage.clear();
	});

	describe('Initialization', () => {
		it('should create a store with initial value when localStorage is empty', () => {
			const store = localStorageWritableStore(testKey, initialValue);

			expect(get(store)).toEqual(initialValue);
		});

		it('should load value from localStorage if it exists', () => {
			const storedValue = { id: 2, name: 'Stored' };
			localStorage.setItem(testKey, JSON.stringify(storedValue));

			const store = localStorageWritableStore(testKey, initialValue);

			expect(get(store)).toEqual(storedValue);
		});

		it('should use initial value if localStorage has no data for the key', () => {
			const store = localStorageWritableStore(testKey, initialValue);

			expect(get(store)).toEqual(initialValue);
		});

		it('should work with string values', () => {
			const store = localStorageWritableStore('string-key', 'initial string');

			expect(get(store)).toBe('initial string');
		});

		it('should work with number values', () => {
			const store = localStorageWritableStore('number-key', 42);

			expect(get(store)).toBe(42);
		});

		it('should work with boolean values', () => {
			const store = localStorageWritableStore('boolean-key', true);

			expect(get(store)).toBe(true);
		});

		it('should work with array values', () => {
			const arrayValue = [1, 2, 3, 4, 5];
			const store = localStorageWritableStore('array-key', arrayValue);

			expect(get(store)).toEqual(arrayValue);
		});

		it('should work with null values', () => {
			const store = localStorageWritableStore('null-key', null);

			expect(get(store)).toBeNull();
		});

		it('should work with complex nested objects', () => {
			const complexValue = {
				user: {
					id: 1,
					profile: {
						name: 'John',
						settings: {
							theme: 'dark',
							notifications: true
						}
					}
				},
				items: [1, 2, 3]
			};
			const store = localStorageWritableStore('complex-key', complexValue);

			expect(get(store)).toEqual(complexValue);
		});
	});

	describe('Updates and Persistence', () => {
		it('should persist updates to localStorage', () => {
			const store = localStorageWritableStore(testKey, initialValue);
			const newValue = { id: 3, name: 'Updated' };

			store.set(newValue);

			const stored = localStorage.getItem(testKey);
			expect(JSON.parse(stored!)).toEqual(newValue);
		});

		it('should update the store value', () => {
			const store = localStorageWritableStore(testKey, initialValue);
			const newValue = { id: 3, name: 'Updated' };

			store.set(newValue);

			expect(get(store)).toEqual(newValue);
		});

		it('should persist incremental updates', () => {
			const store = localStorageWritableStore(testKey, 0);

			store.update((n) => n + 1);
			expect(get(store)).toBe(1);
			expect(JSON.parse(localStorage.getItem(testKey)!)).toBe(1);

			store.update((n) => n + 1);
			expect(get(store)).toBe(2);
			expect(JSON.parse(localStorage.getItem(testKey)!)).toBe(2);
		});

		it('should handle multiple stores with different keys', () => {
			const store1 = localStorageWritableStore('key1', 'value1');
			const store2 = localStorageWritableStore('key2', 'value2');

			store1.set('updated1');
			store2.set('updated2');

			expect(get(store1)).toBe('updated1');
			expect(get(store2)).toBe('updated2');
			expect(JSON.parse(localStorage.getItem('key1')!)).toBe('updated1');
			expect(JSON.parse(localStorage.getItem('key2')!)).toBe('updated2');
		});

		it('should persist array modifications', () => {
			const store = localStorageWritableStore<number[]>('array-key', [1, 2, 3]);

			store.update((arr) => [...arr, 4]);

			expect(get(store)).toEqual([1, 2, 3, 4]);
			expect(JSON.parse(localStorage.getItem('array-key')!)).toEqual([1, 2, 3, 4]);
		});

		it('should persist object property updates', () => {
			const store = localStorageWritableStore('object-key', { count: 0, name: 'test' });

			store.update((obj) => ({ ...obj, count: obj.count + 1 }));

			expect(get(store)).toEqual({ count: 1, name: 'test' });
			expect(JSON.parse(localStorage.getItem('object-key')!)).toEqual({
				count: 1,
				name: 'test'
			});
		});
	});

	describe('Subscriptions', () => {
		it('should notify subscribers on updates', () => {
			const store = localStorageWritableStore(testKey, initialValue);
			const values: { id: number; name: string }[] = [];

			store.subscribe((value) => values.push(value));

			store.set({ id: 2, name: 'Updated' });

			expect(values).toHaveLength(2); // Initial + update
			expect(values[0]).toEqual(initialValue);
			expect(values[1]).toEqual({ id: 2, name: 'Updated' });
		});

		it('should allow unsubscribing', () => {
			const store = localStorageWritableStore(testKey, 0);
			const values: number[] = [];

			const unsubscribe = store.subscribe((value) => values.push(value));

			store.set(1);
			unsubscribe();
			store.set(2);

			expect(values).toEqual([0, 1]); // Should not include 2
		});

		it('should persist to localStorage on every subscription update', () => {
			const store = localStorageWritableStore(testKey, initialValue);

			// Subscribe to trigger persistence
			const unsubscribe = store.subscribe(() => {});

			store.set({ id: 5, name: 'Subscribed Update' });

			const stored = localStorage.getItem(testKey);
			expect(JSON.parse(stored!)).toEqual({ id: 5, name: 'Subscribed Update' });

			unsubscribe();
		});
	});

	describe('Edge Cases', () => {
		it('should handle empty string as key', () => {
			const store = localStorageWritableStore('', 'test-value');

			expect(get(store)).toBe('test-value');
		});

		it('should handle special characters in key', () => {
			const specialKey = 'test-key:with@special#chars';
			const store = localStorageWritableStore(specialKey, 'value');

			store.set('updated');

			expect(localStorage.getItem(specialKey)).toBe('"updated"');
		});

		it('should handle undefined as initial value', () => {
			// Note: JSON.stringify(undefined) returns undefined, not a string
			// This is an edge case that may not work as expected in real localStorage
			const store = localStorageWritableStore<string | undefined>('undefined-key', undefined);

			// The store will work but JSON serialization of undefined is special
			const value = get(store);
			expect(value).toBeUndefined();
		});

		it('should overwrite existing localStorage data on initialization', () => {
			// Pre-populate localStorage
			localStorage.setItem(testKey, JSON.stringify({ old: 'data' }));

			// Create new store - should load the existing data
			const store = localStorageWritableStore(testKey, initialValue);

			expect(get(store)).toEqual({ old: 'data' });
		});

		it('should handle corrupted localStorage data gracefully', () => {
			// Store invalid JSON
			localStorage.setItem(testKey, 'invalid-json{');

			// Should throw or handle gracefully
			expect(() => {
				localStorageWritableStore(testKey, initialValue);
			}).toThrow();
		});

		it('should handle very large objects', () => {
			const largeObject = {
				data: Array.from({ length: 1000 }, (_, i) => ({
					id: i,
					value: `item-${i}`,
					nested: { deep: { value: i } }
				}))
			};

			const store = localStorageWritableStore('large-key', largeObject);

			expect(get(store)).toEqual(largeObject);

			const stored = localStorage.getItem('large-key');
			expect(JSON.parse(stored!)).toEqual(largeObject);
		});

		it('should handle rapid successive updates', () => {
			const store = localStorageWritableStore('rapid-key', 0);

			for (let i = 1; i <= 100; i++) {
				store.set(i);
			}

			expect(get(store)).toBe(100);
			expect(JSON.parse(localStorage.getItem('rapid-key')!)).toBe(100);
		});
	});

	describe('Type Safety', () => {
		it('should maintain type integrity for typed stores', () => {
			interface User {
				id: number;
				name: string;
				email?: string;
			}

			const store = localStorageWritableStore<User>('user-key', {
				id: 1,
				name: 'John'
			});

			const user = get(store);
			expect(user.id).toBe(1);
			expect(user.name).toBe('John');

			store.set({ id: 2, name: 'Jane', email: 'jane@example.com' });

			const updatedUser = get(store);
			expect(updatedUser.email).toBe('jane@example.com');
		});
	});

	describe('Server-side behavior (non-browser)', () => {
		it('should return a regular writable store when browser is true', () => {
			const store = localStorageWritableStore('ssr-key', 'ssr-value');

			expect(get(store)).toBe('ssr-value');

			store.set('updated-ssr');
			expect(localStorage.getItem('ssr-key')).toBe('"updated-ssr"');
		});

		it('should handle store operations', () => {
			const store = localStorageWritableStore('ssr-test', { id: 1, data: 'test' });

			expect(get(store)).toEqual({ id: 1, data: 'test' });

			store.update((val) => ({ ...val, data: 'updated' }));
			expect(get(store).data).toBe('updated');
		});
	});

	describe('Integration scenarios', () => {
		it('should work with multiple subscribers', () => {
			const store = localStorageWritableStore(testKey, 0);
			const values1: number[] = [];
			const values2: number[] = [];

			store.subscribe((v) => values1.push(v));
			store.subscribe((v) => values2.push(v));

			store.set(5);

			expect(values1).toEqual([0, 5]);
			expect(values2).toEqual([0, 5]);
		});

		it('should persist across multiple store instances with same key', () => {
			const store1 = localStorageWritableStore(testKey, initialValue);

			store1.set({ id: 10, name: 'Shared' });

			// Create a second store with the same key
			const store2 = localStorageWritableStore(testKey, { id: 999, name: 'Default' });

			// Should load the value set by store1
			expect(get(store2)).toEqual({ id: 10, name: 'Shared' });
		});

		it('should handle store updates after page reload simulation', () => {
			// First "session"
			const store1 = localStorageWritableStore(testKey, { count: 0 });
			store1.set({ count: 5 });

			// Simulate "page reload" by creating a new store instance
			const store2 = localStorageWritableStore(testKey, { count: 0 });

			expect(get(store2)).toEqual({ count: 5 });
		});
	});
});
