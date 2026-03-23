import { describe, it, expect, vi, afterEach } from 'vitest';
import { formatSeeders, getSeedersColor, formatSearchSize, formatUploadDate } from '../src/format';

describe('formatSeeders', () => {
	it('returns count as string for small numbers', () => {
		expect(formatSeeders(5)).toBe('5');
		expect(formatSeeders(999)).toBe('999');
	});

	it('formats thousands with k suffix', () => {
		expect(formatSeeders(1000)).toBe('1.0k');
		expect(formatSeeders(2500)).toBe('2.5k');
	});

	it('handles zero', () => {
		expect(formatSeeders(0)).toBe('0');
	});
});

describe('getSeedersColor', () => {
	it('returns success for >= 50 seeders', () => {
		expect(getSeedersColor(50)).toBe('text-success');
		expect(getSeedersColor(100)).toBe('text-success');
	});

	it('returns warning for 10-49 seeders', () => {
		expect(getSeedersColor(10)).toBe('text-warning');
		expect(getSeedersColor(49)).toBe('text-warning');
	});

	it('returns error for 1-9 seeders', () => {
		expect(getSeedersColor(1)).toBe('text-error');
		expect(getSeedersColor(9)).toBe('text-error');
	});

	it('returns muted for zero seeders', () => {
		expect(getSeedersColor(0)).toBe('text-base-content/40');
	});
});

describe('formatSearchSize', () => {
	it('formats zero bytes', () => {
		expect(formatSearchSize(0)).toBe('0 B');
	});

	it('formats bytes', () => {
		expect(formatSearchSize(500)).toBe('500 B');
	});

	it('formats kilobytes', () => {
		expect(formatSearchSize(1024)).toBe('1.0 KB');
	});

	it('formats megabytes', () => {
		expect(formatSearchSize(1048576)).toBe('1.0 MB');
	});

	it('formats gigabytes', () => {
		expect(formatSearchSize(1073741824)).toBe('1.0 GB');
	});

	it('formats with decimal precision', () => {
		expect(formatSearchSize(1536 * 1024)).toBe('1.5 MB');
	});
});

describe('formatUploadDate', () => {
	afterEach(() => {
		vi.useRealTimers();
	});

	it('returns Today for same day', () => {
		const now = new Date();
		expect(formatUploadDate(now)).toBe('Today');
	});

	it('returns Yesterday for one day ago', () => {
		const yesterday = new Date(Date.now() - 86400000);
		expect(formatUploadDate(yesterday)).toBe('Yesterday');
	});

	it('returns days ago for less than 30 days', () => {
		const daysAgo = new Date(Date.now() - 86400000 * 15);
		expect(formatUploadDate(daysAgo)).toBe('15d ago');
	});

	it('returns months ago for less than 365 days', () => {
		const monthsAgo = new Date(Date.now() - 86400000 * 60);
		expect(formatUploadDate(monthsAgo)).toBe('2mo ago');
	});

	it('returns years ago for 365+ days', () => {
		const yearsAgo = new Date(Date.now() - 86400000 * 400);
		expect(formatUploadDate(yearsAgo)).toBe('1y ago');
	});
});
