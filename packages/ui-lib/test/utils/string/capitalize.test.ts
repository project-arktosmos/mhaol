import { describe, it, expect } from 'vitest';
import capitalize from '../../../src/utils/string/capitalize';

describe('capitalize', () => {
	it('returns empty string for null', () => {
		expect(capitalize(null)).toBe('');
	});

	it('returns empty string for undefined', () => {
		expect(capitalize(undefined)).toBe('');
	});

	it('returns empty string for empty string', () => {
		expect(capitalize('')).toBe('');
	});

	it('capitalizes a single word', () => {
		expect(capitalize('hello')).toBe('Hello');
	});

	it('capitalizes multiple words', () => {
		expect(capitalize('hello world')).toBe('Hello World');
	});

	it('splits camelCase into separate words', () => {
		expect(capitalize('camelCase')).toBe('Camel Case');
	});

	it('replaces dashes with spaces and capitalizes', () => {
		expect(capitalize('some-thing')).toBe('Some Thing');
	});

	it('lowercases already uppercase words', () => {
		expect(capitalize('HELLO WORLD')).toBe('Hello World');
	});

	it('handles mixed camelCase with dashes', () => {
		expect(capitalize('myComponent-name')).toBe('My Component Name');
	});
});
