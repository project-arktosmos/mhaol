import type { ID } from '../../src/types/core.type';

/**
 * Helper function to create mock items with IDs for testing
 */
export function createMockItem<T extends Record<string, unknown>>(
	id: ID,
	data: Partial<T> = {}
): T & { id: ID } {
	return {
		id,
		...data
	} as T & { id: ID };
}

/**
 * Helper to create multiple mock items
 */
export function createMockItems<T extends Record<string, unknown>>(
	count: number,
	baseData: Partial<T> = {}
): Array<T & { id: ID }> {
	return Array.from({ length: count }, (_, i) => createMockItem<T>(i + 1, baseData));
}

/**
 * Helper to get localStorage data parsed as JSON
 */
export function getLocalStorageItem<T>(key: string): T | null {
	const item = localStorage.getItem(key);
	return item ? JSON.parse(item) : null;
}

/**
 * Helper to set localStorage data as JSON
 */
export function setLocalStorageItem<T>(key: string, value: T): void {
	localStorage.setItem(key, JSON.stringify(value));
}

/**
 * Wait for async operations to complete
 */
export function waitFor(ms: number): Promise<void> {
	return new Promise((resolve) => setTimeout(resolve, ms));
}
