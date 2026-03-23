import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { get } from 'svelte/store';

describe('ClientIdentityService', () => {
	let clientIdentityService: (typeof import('../../src/services/client-identity.service'))['clientIdentityService'];

	beforeEach(async () => {
		vi.resetModules();
		localStorage.clear();
		const mod = await import('../../src/services/client-identity.service');
		clientIdentityService = mod.clientIdentityService;
	});

	afterEach(() => {
		vi.restoreAllMocks();
	});

	// ===== Initial state =====

	it('has correct initial state', () => {
		const state = get(clientIdentityService.state);
		expect(state.loading).toBe(true);
		expect(state.identity).toBeNull();
		expect(state.error).toBeNull();
	});

	// ===== initialize =====

	it('initialize generates a new identity when none is stored', async () => {
		await clientIdentityService.initialize('wss://signaling.test');

		const state = get(clientIdentityService.state);
		expect(state.loading).toBe(false);
		expect(state.error).toBeNull();
		expect(state.identity).not.toBeNull();
		expect(state.identity!.name).toBe('default');
		expect(state.identity!.address).toMatch(/^0x[0-9a-fA-F]{40}$/);
		expect(state.identity!.passport).toBeDefined();
		expect(state.identity!.passport.raw).toBeDefined();
		expect(state.identity!.passport.hash).toBeDefined();
		expect(state.identity!.passport.signature).toBeDefined();
	});

	it('initialize stores the key in localStorage', async () => {
		await clientIdentityService.initialize('wss://signaling.test');

		const stored = localStorage.getItem('client-identity');
		expect(stored).not.toBeNull();

		const parsed = JSON.parse(stored!);
		expect(parsed.name).toBe('default');
		expect(parsed.privateKey).toMatch(/^0x[0-9a-f]{64}$/);
	});

	it('initialize restores identity from localStorage', async () => {
		// First initialize to generate and store
		await clientIdentityService.initialize('wss://signaling.test');
		const firstState = get(clientIdentityService.state);
		const firstAddress = firstState.identity!.address;

		// Re-import to get a fresh service
		vi.resetModules();
		const mod2 = await import('../../src/services/client-identity.service');
		const service2 = mod2.clientIdentityService;

		await service2.initialize('wss://signaling.test');

		const state = get(service2.state);
		expect(state.identity!.address).toBe(firstAddress);
	});

	it('initialize is idempotent', async () => {
		await clientIdentityService.initialize('wss://signaling.test');
		const firstState = get(clientIdentityService.state);

		await clientIdentityService.initialize('wss://other.test');
		const secondState = get(clientIdentityService.state);

		expect(secondState.identity!.address).toBe(firstState.identity!.address);
	});

	it('initialize sets error on failure', async () => {
		// Store invalid JSON to trigger parse error
		localStorage.setItem('client-identity', 'not-json');

		await clientIdentityService.initialize('wss://signaling.test');

		const state = get(clientIdentityService.state);
		expect(state.loading).toBe(false);
		expect(state.identity).toBeNull();
		expect(state.error).not.toBeNull();
	});

	it('initialize includes signalingUrl in passport raw data', async () => {
		await clientIdentityService.initialize('wss://signaling.custom');

		const state = get(clientIdentityService.state);
		const rawData = JSON.parse(state.identity!.passport.raw);
		expect(rawData.signalingUrl).toBe('wss://signaling.custom');
		expect(rawData.instanceType).toBe('client');
	});

	// ===== signMessage =====

	it('signMessage returns a valid signature', async () => {
		await clientIdentityService.initialize('wss://signaling.test');

		const signature = await clientIdentityService.signMessage('hello world');
		expect(signature).toMatch(/^0x[0-9a-f]+$/);
	});

	it('signMessage throws when no identity is stored', async () => {
		// Do not initialize, so no identity in localStorage
		await expect(clientIdentityService.signMessage('test')).rejects.toThrow(
			'No identity initialized'
		);
	});

	it('signMessage produces consistent signatures for same message', async () => {
		await clientIdentityService.initialize('wss://signaling.test');

		const sig1 = await clientIdentityService.signMessage('same message');
		const sig2 = await clientIdentityService.signMessage('same message');
		expect(sig1).toBe(sig2);
	});

	it('signMessage produces different signatures for different messages', async () => {
		await clientIdentityService.initialize('wss://signaling.test');

		const sig1 = await clientIdentityService.signMessage('message one');
		const sig2 = await clientIdentityService.signMessage('message two');
		expect(sig1).not.toBe(sig2);
	});

	// ===== regenerate =====

	it('regenerate creates a new identity with a different address', async () => {
		await clientIdentityService.initialize('wss://signaling.test');
		const firstAddress = get(clientIdentityService.state).identity!.address;

		await clientIdentityService.regenerate();

		const state = get(clientIdentityService.state);
		expect(state.identity).not.toBeNull();
		expect(state.identity!.address).toMatch(/^0x[0-9a-fA-F]{40}$/);
		// Extremely unlikely to get the same address
		expect(state.identity!.address).not.toBe(firstAddress);
	});

	it('regenerate updates localStorage', async () => {
		await clientIdentityService.initialize('wss://signaling.test');
		const storedBefore = localStorage.getItem('client-identity');

		await clientIdentityService.regenerate();
		const storedAfter = localStorage.getItem('client-identity');

		expect(storedAfter).not.toBe(storedBefore);
	});

	it('regenerate sets loading false and no error', async () => {
		await clientIdentityService.initialize('wss://signaling.test');
		await clientIdentityService.regenerate();

		const state = get(clientIdentityService.state);
		expect(state.loading).toBe(false);
		expect(state.error).toBeNull();
	});

	it('regenerate sets name to default', async () => {
		await clientIdentityService.initialize('wss://signaling.test');
		await clientIdentityService.regenerate();

		const state = get(clientIdentityService.state);
		expect(state.identity!.name).toBe('default');
	});

	// ===== Passport data structure =====

	it('passport raw contains expected fields', async () => {
		await clientIdentityService.initialize('wss://signaling.test');

		const state = get(clientIdentityService.state);
		const raw = JSON.parse(state.identity!.passport.raw);
		expect(raw).toHaveProperty('name', 'default');
		expect(raw).toHaveProperty('address');
		expect(raw).toHaveProperty('instanceType', 'client');
		expect(raw).toHaveProperty('signalingUrl', 'wss://signaling.test');
	});

	it('passport hash is a valid hex string', async () => {
		await clientIdentityService.initialize('wss://signaling.test');

		const state = get(clientIdentityService.state);
		expect(state.identity!.passport.hash).toMatch(/^0x[0-9a-f]+$/);
	});

	it('passport signature is a valid hex string', async () => {
		await clientIdentityService.initialize('wss://signaling.test');

		const state = get(clientIdentityService.state);
		expect(state.identity!.passport.signature).toMatch(/^0x[0-9a-f]+$/);
	});
});
