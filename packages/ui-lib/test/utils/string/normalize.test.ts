import { describe, it, expect } from 'vitest';
import normalize from '../../../src/utils/string/normalize';

describe('normalize', () => {
	it('converts to lowercase kebab-case', () => {
		expect(normalize('Hello World')).toBe('hello-world');
	});

	it('strips HTML tags', () => {
		expect(normalize('<b>bold</b> text')).toBe('bold-text');
	});

	it('trims whitespace', () => {
		expect(normalize('  hello  ')).toBe('hello');
	});

	it('handles multiple spaces', () => {
		expect(normalize('hello   world')).toBe('hello---world');
	});

	it('handles empty string', () => {
		expect(normalize('')).toBe('');
	});

	it('handles string with only HTML', () => {
		expect(normalize('<br/>')).toBe('');
	});
});
