import { browser } from '$app/environment';
import { apiUrl } from './api-base';

let installed = false;

// Wraps `globalThis.fetch` so any request whose URL starts with `/api/` is
// rewritten through `apiUrl(...)`. Many services in the SPA call `fetch('/api/…')`
// directly (rather than going through the transport layer); on Tauri shells the
// SPA is served from `tauri://localhost`, so a relative `/api/…` would never
// reach the backend. The interceptor lets every existing call site keep working
// without per-file changes whenever the configured backend URL is non-empty.
export function installFetchInterceptor(): void {
	if (!browser || installed) return;
	installed = true;

	const originalFetch: typeof globalThis.fetch = globalThis.fetch.bind(globalThis);

	const rewriteString = (url: string): string => (url.startsWith('/api/') ? apiUrl(url) : url);

	globalThis.fetch = ((input: RequestInfo | URL, init?: RequestInit): Promise<Response> => {
		if (typeof input === 'string') {
			return originalFetch(rewriteString(input), init);
		}
		if (input instanceof URL) {
			if (input.origin === window.location.origin && input.pathname.startsWith('/api/')) {
				return originalFetch(apiUrl(input.pathname + input.search + input.hash), init);
			}
			return originalFetch(input, init);
		}
		// Request object
		if (input.url) {
			try {
				const u = new URL(input.url, window.location.href);
				if (u.origin === window.location.origin && u.pathname.startsWith('/api/')) {
					const next = apiUrl(u.pathname + u.search + u.hash);
					return originalFetch(new Request(next, input), init);
				}
			} catch {
				/* ignore parse errors and fall through */
			}
		}
		return originalFetch(input, init);
	}) as typeof globalThis.fetch;

	// EventSource has no convenient interception hook; the only `new EventSource(...)`
	// site is the transport layer, which already routes through `apiUrl`.
}
