import { describe, it, expect } from 'vitest';
import { P2pAdapter } from '../../src/adapters/classes/p2p.adapter';
import type { P2pConnectionPhase } from '../../src/types/p2p.type';

describe('P2pAdapter', () => {
	const adapter = new P2pAdapter();

	describe('Initialization', () => {
		it('should have correct adapter id', () => {
			expect(adapter.id).toBe('adapter:p2p');
		});
	});

	describe('shortAddress', () => {
		it('should shorten valid Ethereum addresses', () => {
			expect(adapter.shortAddress('0x1234567890abcdef1234567890abcdef12345678')).toBe(
				'0x1234...5678'
			);
		});

		it('should return short strings as-is', () => {
			expect(adapter.shortAddress('short')).toBe('short');
		});

		it('should return non-0x strings as-is', () => {
			expect(adapter.shortAddress('1234567890abcdef1234567890abcdef12345678')).toBe(
				'1234567890abcdef1234567890abcdef12345678'
			);
		});

		it('should handle addresses exactly at the boundary', () => {
			expect(adapter.shortAddress('0x12345678')).toBe('0x1234...5678');
		});
	});

	describe('phaseLabel', () => {
		it('should return correct labels for all phases', () => {
			const phases: P2pConnectionPhase[] = [
				'idle',
				'creating-offer',
				'waiting-answer',
				'accepting-offer',
				'answer-ready',
				'connecting',
				'connected',
				'disconnected',
				'error'
			];

			for (const phase of phases) {
				const label = adapter.phaseLabel(phase);
				expect(label).toBeTruthy();
				expect(typeof label).toBe('string');
			}
		});

		it('should return "Not connected" for idle', () => {
			expect(adapter.phaseLabel('idle')).toBe('Not connected');
		});

		it('should return "Connected" for connected', () => {
			expect(adapter.phaseLabel('connected')).toBe('Connected');
		});
	});

	describe('phaseBadgeClass', () => {
		it('should return badge-success for connected', () => {
			expect(adapter.phaseBadgeClass('connected')).toBe('badge-success');
		});

		it('should return badge-error for error', () => {
			expect(adapter.phaseBadgeClass('error')).toBe('badge-error');
		});

		it('should return badge-ghost for idle', () => {
			expect(adapter.phaseBadgeClass('idle')).toBe('badge-ghost');
		});
	});

	describe('createMessage', () => {
		it('should create a message with required fields', () => {
			const msg = adapter.createMessage('0xABC', 'Hello');
			expect(msg.address).toBe('0xABC');
			expect(msg.content).toBe('Hello');
			expect(msg.id).toBeDefined();
			expect(msg.timestamp).toBeDefined();
		});

		it('should create unique IDs for each message', () => {
			const msg1 = adapter.createMessage('0xABC', 'Hello');
			const msg2 = adapter.createMessage('0xABC', 'World');
			expect(msg1.id).not.toBe(msg2.id);
		});

		it('should create valid ISO timestamps', () => {
			const msg = adapter.createMessage('0xABC', 'Hello');
			const parsed = new Date(msg.timestamp);
			expect(parsed.getTime()).not.toBeNaN();
		});
	});

	describe('formatTimestamp', () => {
		it('should format an ISO timestamp', () => {
			const result = adapter.formatTimestamp('2026-01-15T14:30:00.000Z');
			expect(typeof result).toBe('string');
			expect(result.length).toBeGreaterThan(0);
		});
	});
});
