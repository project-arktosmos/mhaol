import { describe, it, expect } from 'vitest';
import { signalingAdapter } from '../../src/adapters/classes/signaling.adapter';

describe('SignalingAdapter', () => {
	describe('shortAddress', () => {
		it('shortens a hex address', () => {
			expect(signalingAdapter.shortAddress('0x1234567890abcdef')).toBe('0x1234...cdef');
		});

		it('returns short addresses unchanged', () => {
			expect(signalingAdapter.shortAddress('0x12')).toBe('0x12');
		});

		it('returns non-hex addresses unchanged', () => {
			expect(signalingAdapter.shortAddress('hello')).toBe('hello');
		});
	});

	describe('formatTimestamp', () => {
		it('formats an ISO timestamp', () => {
			const result = signalingAdapter.formatTimestamp('2023-01-15T10:30:00Z');
			expect(result).toMatch(/\d{1,2}:\d{2}/);
		});
	});

	describe('phaseLabel', () => {
		it('maps phases to labels', () => {
			expect(signalingAdapter.phaseLabel('disconnected')).toBe('Disconnected');
			expect(signalingAdapter.phaseLabel('connecting')).toBe('Connecting...');
			expect(signalingAdapter.phaseLabel('authenticated')).toBe('Authenticated');
			expect(signalingAdapter.phaseLabel('connected')).toBe('Connected');
			expect(signalingAdapter.phaseLabel('error')).toBe('Error');
		});
	});

	describe('phaseBadgeClass', () => {
		it('maps phases to badge classes', () => {
			expect(signalingAdapter.phaseBadgeClass('disconnected')).toBe('badge-ghost');
			expect(signalingAdapter.phaseBadgeClass('connected')).toBe('badge-success');
			expect(signalingAdapter.phaseBadgeClass('error')).toBe('badge-error');
		});
	});

	describe('createMessage', () => {
		it('creates a chat message with all fields', () => {
			const msg = signalingAdapter.createMessage('0xabc', 'hello');
			expect(msg.address).toBe('0xabc');
			expect(msg.content).toBe('hello');
			expect(msg.id).toBeTruthy();
			expect(msg.timestamp).toBeTruthy();
		});
	});

	describe('playerConnectionLabel', () => {
		it('maps states to labels', () => {
			expect(signalingAdapter.playerConnectionLabel('idle')).toBe('Idle');
			expect(signalingAdapter.playerConnectionLabel('streaming')).toBe('Streaming');
			expect(signalingAdapter.playerConnectionLabel('error')).toBe('Error');
		});
	});

	describe('playerConnectionBadgeClass', () => {
		it('maps states to badge classes', () => {
			expect(signalingAdapter.playerConnectionBadgeClass('idle')).toBe('badge-ghost');
			expect(signalingAdapter.playerConnectionBadgeClass('streaming')).toBe('badge-success');
			expect(signalingAdapter.playerConnectionBadgeClass('error')).toBe('badge-error');
		});
	});

	describe('resolveLocalUrl', () => {
		it('returns non-local URLs unchanged', () => {
			expect(signalingAdapter.resolveLocalUrl('https://example.com/path')).toBe(
				'https://example.com/path'
			);
		});

		it('handles invalid URLs gracefully', () => {
			expect(signalingAdapter.resolveLocalUrl('not-a-url')).toBe('not-a-url');
		});

		it('preserves the rendezvous port when the browser is on localhost', () => {
			// Regression: previously rewrote both hostname and port with
			// window.location, sending the player to the cloud port (9898)
			// instead of the rendezvous port (14080).
			Object.defineProperty(window, 'location', {
				configurable: true,
				value: new URL('http://localhost:9898/')
			});
			expect(signalingAdapter.resolveLocalUrl('http://localhost:14080')).toBe(
				'http://localhost:14080'
			);
		});

		it('rewrites hostname but keeps port when the browser is on a different host', () => {
			// Cross-machine case: cloud advertises its loopback URL, browser
			// sits on a real LAN IP. We swap the hostname so the browser
			// dials the cloud's host on the rendezvous port.
			Object.defineProperty(window, 'location', {
				configurable: true,
				value: new URL('http://10.0.0.5:9898/')
			});
			expect(signalingAdapter.resolveLocalUrl('http://localhost:14080')).toBe(
				'http://10.0.0.5:14080'
			);
		});

		it('does not rewrite when both sides are loopback', () => {
			Object.defineProperty(window, 'location', {
				configurable: true,
				value: new URL('http://127.0.0.1:9898/')
			});
			expect(signalingAdapter.resolveLocalUrl('http://127.0.0.1:14080')).toBe(
				'http://127.0.0.1:14080'
			);
		});
	});

	describe('buildWsUrl', () => {
		it('builds ws URL from http base', () => {
			const result = signalingAdapter.buildWsUrl('http://localhost:1530', 'room1');
			expect(result).toBe('ws://localhost:1530/party/room1');
		});

		it('builds wss URL from https base', () => {
			const result = signalingAdapter.buildWsUrl('https://example.com', 'room2');
			expect(result).toBe('wss://example.com/party/room2');
		});
	});
});
