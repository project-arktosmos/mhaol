import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { get } from 'svelte/store';
import { walletService } from '../../src/services/wallet.service';

const mockWallet = { name: 'TestWallet', address: '0xabc123' };

function mockFetchSuccess(data: unknown = mockWallet) {
	return vi.fn().mockResolvedValue({
		ok: true,
		json: () => Promise.resolve(data)
	});
}

function mockFetchError(status = 500) {
	return vi.fn().mockResolvedValue({
		ok: false,
		status
	});
}

describe('walletService', () => {
	beforeEach(() => {
		walletService.state.set({
			loading: false,
			name: null,
			address: null,
			error: null
		});
		(walletService as any)._initialized = false;
	});

	afterEach(() => {
		vi.restoreAllMocks();
	});

	it('should have empty initial state', () => {
		const state = get(walletService.state);
		expect(state.name).toBeNull();
		expect(state.address).toBeNull();
		expect(state.loading).toBe(false);
		expect(state.error).toBeNull();
	});

	it('should fetch wallet on initialize', async () => {
		vi.stubGlobal('fetch', mockFetchSuccess());

		await walletService.initialize();

		const state = get(walletService.state);
		expect(state.name).toBe('TestWallet');
		expect(state.address).toBe('0xabc123');
		expect(state.loading).toBe(false);
	});

	it('should not re-initialize if already initialized', async () => {
		const fetchMock = mockFetchSuccess();
		vi.stubGlobal('fetch', fetchMock);

		await walletService.initialize();
		await walletService.initialize();

		expect(fetchMock).toHaveBeenCalledTimes(1);
	});

	it('should handle fetch error on initialize', async () => {
		vi.stubGlobal('fetch', mockFetchError(500));

		await walletService.initialize();

		const state = get(walletService.state);
		expect(state.error).toBe('HTTP 500');
		expect(state.loading).toBe(false);
	});

	it('should regenerate wallet', async () => {
		const newWallet = { name: 'NewWallet', address: '0xdef456' };
		vi.stubGlobal('fetch', mockFetchSuccess(newWallet));

		await walletService.regenerate();

		const state = get(walletService.state);
		expect(state.name).toBe('NewWallet');
		expect(state.address).toBe('0xdef456');
	});

	it('should sign a message and return signature', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn().mockResolvedValue({
				ok: true,
				json: () => Promise.resolve({ signature: '0xsigned' })
			})
		);

		const signature = await walletService.sign('hello');
		expect(signature).toBe('0xsigned');
	});

	it('should return null on sign failure', async () => {
		vi.stubGlobal('fetch', mockFetchError(500));

		const signature = await walletService.sign('hello');
		expect(signature).toBeNull();
	});

	it('should return address via getAddress', async () => {
		vi.stubGlobal('fetch', mockFetchSuccess());
		await walletService.initialize();

		expect(walletService.getAddress()).toBe('0xabc123');
	});
});
