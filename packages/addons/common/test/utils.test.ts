import { describe, it, expect } from 'vitest';
import { extractYear } from '../src/utils.js';

describe('extractYear', () => {
	it('extracts year from a date string', () => {
		expect(extractYear('2023-05-15')).toBe('2023');
	});

	it('returns year from year-only string', () => {
		expect(extractYear('2020')).toBe('2020');
	});

	it('returns Unknown for undefined', () => {
		expect(extractYear(undefined)).toBe('Unknown');
	});

	it('returns Unknown for empty string', () => {
		expect(extractYear('')).toBe('Unknown');
	});

	it('handles date with different separators', () => {
		expect(extractYear('1999-12-31')).toBe('1999');
	});
});
