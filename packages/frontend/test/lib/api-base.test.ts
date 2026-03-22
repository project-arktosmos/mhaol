import { describe, it, expect } from 'vitest';

describe('api-base', () => {
	it('exports apiUrl function', async () => {
		const { apiUrl } = await import('../../src/lib/api-base');
		expect(typeof apiUrl).toBe('function');
	});

	it('apiUrl prepends base to path', async () => {
		const { apiUrl } = await import('../../src/lib/api-base');
		const result = apiUrl('/api/test');
		expect(result).toContain('/api/test');
	});

	it('exports apiBase string', async () => {
		const { apiBase } = await import('../../src/lib/api-base');
		expect(typeof apiBase).toBe('string');
	});
});
