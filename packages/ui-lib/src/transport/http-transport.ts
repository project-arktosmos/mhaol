import { apiUrl } from 'ui-lib/lib/api-base';
import type {
	Transport,
	TransportRequestInit,
	TransportResponse,
	TransportEventSource
} from './transport.type';
import type { AuthProvider } from './passport-auth';

export class HttpTransport implements Transport {
	private authProvider: AuthProvider | null;

	constructor(authProvider?: AuthProvider) {
		this.authProvider = authProvider ?? null;
	}

	async fetch(path: string, init?: TransportRequestInit): Promise<TransportResponse> {
		const authHeaders = this.authProvider ? await this.authProvider.getAuthHeaders() : {};

		const response = await globalThis.fetch(apiUrl(path), {
			method: init?.method ?? 'GET',
			headers: {
				...authHeaders,
				...init?.headers
			},
			body: init?.body,
			signal: init?.signal
		});

		return {
			ok: response.ok,
			status: response.status,
			statusText: response.statusText,
			json: () => response.json(),
			text: () => response.text(),
			body: response.body,
			headers: { get: (name: string) => response.headers.get(name) }
		};
	}

	subscribe(path: string, options?: { signal?: AbortSignal }): TransportEventSource {
		return new HttpEventSource(apiUrl(path), options?.signal, this.authProvider);
	}

	resolveUrl(path: string): string {
		return apiUrl(path);
	}

	async resolveUrlAsync(path: string): Promise<string> {
		return apiUrl(path);
	}
}

class HttpEventSource implements TransportEventSource {
	private es: EventSource | null = null;
	private listeners = new Map<string, Map<(data: string) => void, (e: MessageEvent) => void>>();
	private pendingListeners: { type: string; callback: (data: string) => void }[] = [];

	constructor(url: string, signal?: AbortSignal, authProvider?: AuthProvider | null) {
		if (authProvider) {
			// EventSource doesn't support custom headers, so we append auth as query params
			authProvider.getAuthQueryParams().then((params) => {
				const separator = url.includes('?') ? '&' : '?';
				this.es = new EventSource(`${url}${separator}${params}`);
				signal?.addEventListener('abort', () => this.close());
				// Attach any listeners that were added before the EventSource was ready
				for (const { type, callback } of this.pendingListeners) {
					this.addListenerToEs(type, callback);
				}
				this.pendingListeners = [];
			});
		} else {
			this.es = new EventSource(url);
			signal?.addEventListener('abort', () => this.close());
		}
	}

	addEventListener(type: string, callback: (data: string) => void): void {
		if (this.es) {
			this.addListenerToEs(type, callback);
		} else {
			this.pendingListeners.push({ type, callback });
		}
	}

	removeEventListener(type: string, callback: (data: string) => void): void {
		const typeListeners = this.listeners.get(type);
		if (!typeListeners) return;
		const wrapper = typeListeners.get(callback);
		if (wrapper) {
			this.es?.removeEventListener(type, wrapper);
			typeListeners.delete(callback);
		}
	}

	close(): void {
		this.es?.close();
	}

	private addListenerToEs(type: string, callback: (data: string) => void): void {
		const wrapper = (e: MessageEvent) => callback(e.data);
		if (!this.listeners.has(type)) {
			this.listeners.set(type, new Map());
		}
		this.listeners.get(type)!.set(callback, wrapper);
		this.es!.addEventListener(type, wrapper);
	}
}
