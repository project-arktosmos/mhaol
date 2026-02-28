import type { PluginCompanion } from '../types';
import { YouTubeDownloadRepository } from 'database/repositories';
import { getPoToken, refreshPoToken } from '$lib/server/po-token';

async function waitForServer(
	baseUrl: string,
	healthPath: string,
	retries: number,
	intervalMs: number
): Promise<boolean> {
	for (let i = 0; i < retries; i++) {
		try {
			const res = await fetch(`${baseUrl}${healthPath}`);
			if (res.ok) return true;
		} catch {
			// Server not ready yet
		}
		await new Promise((r) => setTimeout(r, intervalMs));
	}
	return false;
}

async function syncConfigToRust(
	baseUrl: string,
	config: Record<string, string>
): Promise<void> {
	await fetch(`${baseUrl}/api/config`, {
		method: 'PUT',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify(config)
	});
}

export const ytDownloadCompanion: PluginCompanion = {
	repositories: [{ class: YouTubeDownloadRepository, localsKey: 'youtubeDownloadRepo' }],

	async onInit(ctx) {
		const baseUrl = ctx.getProcessUrl('ytdl-server');
		if (!ctx.isProcessAvailable('ytdl-server')) return;

		const persistedPoToken = ctx.settingsRepo.get('youtube.poToken') ?? undefined;
		const persistedCookies = ctx.settingsRepo.get('youtube.cookies') ?? undefined;

		// If we already have auth tokens, sync them to the Rust server once it's ready
		if (persistedPoToken || persistedCookies) {
			const ready = await waitForServer(baseUrl, '/api/status', 20, 500);
			if (ready) {
				try {
					await syncConfigToRust(baseUrl, {
						...(persistedPoToken ? { poToken: persistedPoToken } : {}),
						...(persistedCookies ? { cookies: persistedCookies } : {})
					});
				} catch {
					console.warn('[yt-download] Failed to sync auth config to Rust server');
				}
			}
			return;
		}

		// No persisted PO token — auto-generate one
		const ready = await waitForServer(baseUrl, '/api/status', 20, 500);
		if (!ready) {
			console.warn('[po-token] Rust server not ready, skipping auto-generation');
			return;
		}

		try {
			const { poToken, visitorData } = await getPoToken();
			await syncConfigToRust(baseUrl, { poToken, visitorData });
			ctx.settingsRepo.set('youtube.poToken', poToken);
			ctx.settingsRepo.set('youtube.visitorData', visitorData);
			console.log('[po-token] Auto-generated and synced to Rust server');
		} catch (e) {
			console.warn(`[po-token] Failed to auto-generate: ${e}`);
		}
	},

	scheduledTasks: [
		{
			id: 'po-token-refresh',
			intervalMs: 6 * 60 * 60 * 1000,
			async handler(ctx) {
				const baseUrl = ctx.getProcessUrl('ytdl-server');
				if (!ctx.isProcessAvailable('ytdl-server')) return;

				try {
					const { poToken, visitorData } = await refreshPoToken();
					await syncConfigToRust(baseUrl, { poToken, visitorData });
					ctx.settingsRepo.set('youtube.poToken', poToken);
					ctx.settingsRepo.set('youtube.visitorData', visitorData);
					console.log('[po-token] Refreshed and synced to Rust server');
				} catch (e) {
					console.warn(`[po-token] Failed to refresh: ${e}`);
				}
			}
		}
	]
};
