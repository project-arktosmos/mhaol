import { describe, it, expect } from 'vitest';
import { identityAdapter } from '../../src/adapters/classes/identity.adapter';

describe('IdentityAdapter', () => {
	describe('shortAddress', () => {
		it('shortens a long hex address', () => {
			expect(identityAdapter.shortAddress('0x1234567890abcdef1234')).toBe('0x1234...1234');
		});

		it('returns short addresses unchanged', () => {
			expect(identityAdapter.shortAddress('0x1234')).toBe('0x1234');
		});

		it('returns non-0x addresses unchanged', () => {
			expect(identityAdapter.shortAddress('hello-world')).toBe('hello-world');
		});

		it('returns empty string unchanged', () => {
			expect(identityAdapter.shortAddress('')).toBe('');
		});
	});
});
