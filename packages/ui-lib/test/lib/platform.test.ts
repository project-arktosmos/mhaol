import { describe, it, expect } from 'vitest';

describe('platform', () => {
	it('exports isTauri as a boolean', async () => {
		const { isTauri } = await import('../../src/lib/platform');
		expect(typeof isTauri).toBe('boolean');
	});

	it('exports isMobile as a boolean', async () => {
		const { isMobile } = await import('../../src/lib/platform');
		expect(typeof isMobile).toBe('boolean');
	});

	it('isTauri is false in test environment (no __TAURI__)', async () => {
		const { isTauri } = await import('../../src/lib/platform');
		expect(isTauri).toBe(false);
	});

	it('isMobile is false in test environment', async () => {
		const { isMobile } = await import('../../src/lib/platform');
		expect(isMobile).toBe(false);
	});
});
