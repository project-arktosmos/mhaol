import { isTauri } from 'ui-lib/lib/platform';

const blobUrls = new Map<string, string>();
const inflight = new Map<string, Promise<string>>();

export async function getCachedImageUrl(url: string): Promise<string> {
	if (!url || !isTauri) return url;

	const existing = blobUrls.get(url);
	if (existing) return existing;

	const pending = inflight.get(url);
	if (pending) return pending;

	const promise = (async () => {
		try {
			const { invoke } = await import('@tauri-apps/api/core');
			const buf = await invoke<ArrayBuffer>('image_cache_resolve', { url });
			const objectUrl = URL.createObjectURL(new Blob([buf]));
			blobUrls.set(url, objectUrl);
			return objectUrl;
		} catch (e) {
			console.warn('image cache resolve failed:', url, e);
			return url;
		} finally {
			inflight.delete(url);
		}
	})();

	inflight.set(url, promise);
	return promise;
}
