import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { get } from 'svelte/store';
import { downloadsService } from '../../src/services/downloads.service';

const mockDownloads = [
	{
		id: '1',
		type: 'youtube' as const,
		name: 'Test Video',
		state: 'downloading',
		progress: 50,
		size: 1024,
		outputPath: null,
		error: null,
		createdAt: '2024-01-01',
		updatedAt: '2024-01-01'
	}
];

function mockFetchSuccess(data: unknown = mockDownloads) {
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

describe('downloadsService', () => {
	beforeEach(() => {
		vi.useFakeTimers();
		downloadsService.state.set({
			downloads: [],
			loading: false,
			error: null
		});
		downloadsService.modalOpen.set(false);
		// Reset internal state
		(downloadsService as any).subscribers = 0;
		if ((downloadsService as any).pollTimer) {
			clearInterval((downloadsService as any).pollTimer);
			(downloadsService as any).pollTimer = null;
		}
	});

	afterEach(() => {
		// Clean up polling
		(downloadsService as any).subscribers = 0;
		if ((downloadsService as any).pollTimer) {
			clearInterval((downloadsService as any).pollTimer);
			(downloadsService as any).pollTimer = null;
		}
		vi.useRealTimers();
		vi.restoreAllMocks();
	});

	it('should have empty initial state', () => {
		const state = get(downloadsService.state);
		expect(state.downloads).toEqual([]);
		expect(state.loading).toBe(false);
		expect(state.error).toBeNull();
	});

	it('should open and close modal', () => {
		downloadsService.openModal();
		expect(get(downloadsService.modalOpen)).toBe(true);
		downloadsService.closeModal();
		expect(get(downloadsService.modalOpen)).toBe(false);
	});

	it('should fetch downloads on startPolling', async () => {
		const fetchMock = mockFetchSuccess();
		vi.stubGlobal('fetch', fetchMock);

		downloadsService.startPolling();
		// Flush the initial fetchDownloads promise (not the interval)
		await vi.advanceTimersByTimeAsync(0);

		const state = get(downloadsService.state);
		expect(state.downloads).toHaveLength(1);
		expect(state.downloads[0].name).toBe('Test Video');
		expect(fetchMock).toHaveBeenCalled();
	});

	it('should set error on fetch failure with loading shown', async () => {
		const fetchMock = mockFetchError(500);
		vi.stubGlobal('fetch', fetchMock);

		downloadsService.startPolling();
		await vi.advanceTimersByTimeAsync(0);

		const state = get(downloadsService.state);
		expect(state.error).toBe('HTTP 500');
		expect(state.loading).toBe(false);
	});

	it('should set loading to true initially when polling starts', () => {
		const fetchMock = mockFetchSuccess();
		vi.stubGlobal('fetch', fetchMock);

		downloadsService.startPolling();

		const state = get(downloadsService.state);
		expect(state.loading).toBe(true);
	});

	it('should stop polling when stopPolling is called', async () => {
		const fetchMock = mockFetchSuccess();
		vi.stubGlobal('fetch', fetchMock);

		downloadsService.startPolling();
		await vi.advanceTimersByTimeAsync(0);
		const callCount = fetchMock.mock.calls.length;

		downloadsService.stopPolling();
		await vi.advanceTimersByTimeAsync(10000);

		expect(fetchMock.mock.calls.length).toBe(callCount);
	});
});
