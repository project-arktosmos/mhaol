import { apiUrl } from '$lib/api-base';
import type {
	Transport,
	TransportEventSource,
	TransportRequestInit,
	TransportResponse
} from './transport.type';

// Default fallback used until a real transport (WsTransport) is wired up by
// nodeConnectionService.connectWs. It speaks plain HTTP via globalThis.fetch so
// services can be exercised against mocked fetch in tests and against the
// embedded API server in dev. Not user-selectable: production runtime always
// upgrades to WsTransport at connection time.
class DefaultFetchTransport implements Transport {
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

let _transport: Transport = new DefaultFetchTransport();
let _ready = false;

export function setTransport(transport: Transport): void {
	_transport = transport;
	_ready = true;
}

export function clearTransport(): void {
	_transport = new DefaultFetchTransport();
	_ready = false;
}

export function getTransport(): Transport {
	return _transport;
}

export function isTransportReady(): boolean {
	return _ready;
}
