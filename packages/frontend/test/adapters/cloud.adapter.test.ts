import { describe, it, expect } from 'vitest';
import { cloudAdapter } from '../../src/adapters/classes/cloud.adapter';

describe('CloudAdapter', () => {
	describe('formatBytes', () => {
		it('returns 0 B for null', () => {
			expect(cloudAdapter.formatBytes(null)).toBe('0 B');
		});

		it('returns 0 B for zero', () => {
			expect(cloudAdapter.formatBytes(0)).toBe('0 B');
		});

		it('formats bytes', () => {
			expect(cloudAdapter.formatBytes(500)).toBe('500 B');
		});

		it('formats kilobytes', () => {
			expect(cloudAdapter.formatBytes(1024)).toBe('1.0 KB');
		});

		it('formats megabytes', () => {
			expect(cloudAdapter.formatBytes(1048576)).toBe('1.0 MB');
		});

		it('formats gigabytes', () => {
			expect(cloudAdapter.formatBytes(1073741824)).toBe('1.0 GB');
		});
	});

	describe('extensionBadgeClass', () => {
		it('returns primary for video extensions', () => {
			expect(cloudAdapter.extensionBadgeClass('mp4')).toBe('badge-primary');
			expect(cloudAdapter.extensionBadgeClass('mkv')).toBe('badge-primary');
		});

		it('returns secondary for audio extensions', () => {
			expect(cloudAdapter.extensionBadgeClass('mp3')).toBe('badge-secondary');
			expect(cloudAdapter.extensionBadgeClass('flac')).toBe('badge-secondary');
		});

		it('returns accent for image extensions', () => {
			expect(cloudAdapter.extensionBadgeClass('jpg')).toBe('badge-accent');
			expect(cloudAdapter.extensionBadgeClass('png')).toBe('badge-accent');
		});

		it('returns ghost for unknown extensions', () => {
			expect(cloudAdapter.extensionBadgeClass('txt')).toBe('badge-ghost');
		});
	});

	describe('attributeTypeBadgeClass', () => {
		it('maps types to badge classes', () => {
			expect(cloudAdapter.attributeTypeBadgeClass('string')).toBe('badge-info');
			expect(cloudAdapter.attributeTypeBadgeClass('number')).toBe('badge-success');
			expect(cloudAdapter.attributeTypeBadgeClass('boolean')).toBe('badge-warning');
			expect(cloudAdapter.attributeTypeBadgeClass('date')).toBe('badge-primary');
			expect(cloudAdapter.attributeTypeBadgeClass('url')).toBe('badge-secondary');
			expect(cloudAdapter.attributeTypeBadgeClass('duration')).toBe('badge-accent');
			expect(cloudAdapter.attributeTypeBadgeClass('bytes')).toBe('badge-neutral');
			expect(cloudAdapter.attributeTypeBadgeClass('tags')).toBe('badge-error');
			expect(cloudAdapter.attributeTypeBadgeClass('json')).toBe('badge-ghost');
		});

		it('returns ghost for unknown type', () => {
			expect(cloudAdapter.attributeTypeBadgeClass('other')).toBe('badge-ghost');
		});
	});

	describe('formatAttributeValue', () => {
		it('formats bytes type', () => {
			expect(cloudAdapter.formatAttributeValue('1048576', 'bytes')).toBe('1.0 MB');
		});

		it('returns raw value for invalid bytes', () => {
			expect(cloudAdapter.formatAttributeValue('not-a-number', 'bytes')).toBe('not-a-number');
		});

		it('formats duration type', () => {
			expect(cloudAdapter.formatAttributeValue('125', 'duration')).toBe('2:05');
		});

		it('returns raw value for invalid duration', () => {
			expect(cloudAdapter.formatAttributeValue('invalid', 'duration')).toBe('invalid');
		});

		it('formats boolean type', () => {
			expect(cloudAdapter.formatAttributeValue('true', 'boolean')).toBe('Yes');
			expect(cloudAdapter.formatAttributeValue('1', 'boolean')).toBe('Yes');
			expect(cloudAdapter.formatAttributeValue('false', 'boolean')).toBe('No');
		});

		it('returns raw value for other types', () => {
			expect(cloudAdapter.formatAttributeValue('hello', 'string')).toBe('hello');
		});
	});
});
