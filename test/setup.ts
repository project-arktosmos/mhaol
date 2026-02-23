import '@testing-library/jest-dom/vitest';
import { beforeEach, vi } from 'vitest';

// Mock localStorage
const localStorageMock = (() => {
	let store: Record<string, string> = {};

	return {
		getItem: (key: string) => store[key] || null,
		setItem: (key: string, value: string) => {
			if (value === undefined || value === null) {
				store[key] = String(value);
			} else {
				store[key] = value.toString();
			}
		},
		removeItem: (key: string) => {
			delete store[key];
		},
		clear: () => {
			store = {};
		}
	};
})();

// Mock console methods
global.console = {
	...console,
	log: vi.fn(),
	debug: vi.fn(),
	info: vi.fn(),
	warn: vi.fn(),
	error: vi.fn()
};

Object.defineProperty(window, 'localStorage', {
	value: localStorageMock
});

// Clear mocks and localStorage before each test
beforeEach(() => {
	localStorage.clear();
	vi.clearAllMocks();
});
