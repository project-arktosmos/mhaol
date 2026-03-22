import { describe, it, expect } from 'vitest';
import { llmAdapter } from '../../src/adapters/classes/llm.adapter';

describe('LlmAdapter', () => {
	describe('formatModelSize', () => {
		it('formats gigabytes', () => {
			expect(llmAdapter.formatModelSize(4_294_967_296)).toBe('4.0 GB');
		});

		it('formats megabytes', () => {
			expect(llmAdapter.formatModelSize(52_428_800)).toBe('50 MB');
		});

		it('formats kilobytes', () => {
			expect(llmAdapter.formatModelSize(512_000)).toBe('500 KB');
		});

		it('handles boundary between MB and GB', () => {
			expect(llmAdapter.formatModelSize(1_073_741_824)).toBe('1.0 GB');
		});
	});
});
