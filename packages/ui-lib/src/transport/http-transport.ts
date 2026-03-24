import { apiUrl } from 'ui-lib/lib/api-base';
import type {
	Transport,
	TransportRequestInit,
	TransportResponse,
	TransportEventSource
} from './transport.type';

export class HttpTransport implements Transport {
	async fetch(path: string, init?: TransportRequestInit): Promise<TransportResponse> {
		const response = await globalThis.fetch(apiUrl(path), {
			method: init?.method ?? 'GET',
			headers: init?.headers,
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
		return new HttpEventSource(apiUrl(path), options?.signal);
	}

	resolveUrl(path: string): string {
		return apiUrl(path);
	}

	async resolveUrlAsync(path: string): Promise<string> {
		return apiUrl(path);
	}
}

class HttpEventSource implements TransportEventSource {
	private es: EventSource;
	private listeners = new Map<string, Map<(data: string) => void, (e: MessageEvent) => void>>();

	constructor(url: string, signal?: AbortSignal) {
		this.es = new EventSource(url);
		signal?.addEventListener('abort', () => this.close());
	}

	addEventListener(type: string, callback: (data: string) => void): void {
		const wrapper = (e: MessageEvent) => callback(e.data);
		if (!this.listeners.has(type)) {
			this.listeners.set(type, new Map());
		}
		this.listeners.get(type)!.set(callback, wrapper);
		this.es.addEventListener(type, wrapper);
	}

	removeEventListener(type: string, callback: (data: string) => void): void {
		const typeListeners = this.listeners.get(type);
		if (!typeListeners) return;
		const wrapper = typeListeners.get(callback);
		if (wrapper) {
			this.es.removeEventListener(type, wrapper);
			typeListeners.delete(callback);
		}
	}

	close(): void {
		this.es.close();
	}
}
