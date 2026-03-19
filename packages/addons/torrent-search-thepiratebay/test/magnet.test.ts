import { describe, it, expect } from 'vitest';
import { buildMagnetLink } from '../src/magnet.js';

describe('buildMagnetLink', () => {
	it('generates a valid magnet URI', () => {
		const link = buildMagnetLink('abc123', 'Test Torrent');
		expect(link).toMatch(/^magnet:\?xt=urn:btih:abc123&dn=/);
	});

	it('encodes the torrent name', () => {
		const link = buildMagnetLink('abc123', 'My File (2024)');
		expect(link).toContain('dn=My%20File%20(2024)');
	});

	it('includes tracker URLs', () => {
		const link = buildMagnetLink('abc123', 'Test');
		expect(link).toContain('&tr=');
		expect(link).toContain('tracker.opentrackr.org');
		expect(link).toContain('tracker.openbittorrent.com');
	});

	it('handles special characters in names', () => {
		const link = buildMagnetLink('abc123', 'File & Name + Special = Chars');
		expect(link).toContain('dn=File%20%26%20Name%20%2B%20Special%20%3D%20Chars');
	});
});
