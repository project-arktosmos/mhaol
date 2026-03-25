import { getTransport } from './transport-context';
import type {
	TransportRequestInit,
	TransportResponse,
	TransportEventSource
} from './transport.type';

export async function fetchJson<T>(path: string, init?: RequestInit): Promise<T> {
	const transport = getTransport();
	const response = await transport.fetch(path, {
		method: (init?.method as TransportRequestInit['method']) ?? 'GET',
		headers: {
			'Content-Type': 'application/json',
			...(init?.headers as Record<string, string>)
		},
		body: init?.body as string,
		signal: init?.signal ?? undefined
	});

	if (!response.ok) {
		const body = await response.json().catch(() => ({}));
		throw new Error((body as { error?: string }).error ?? `HTTP ${response.status}`);
	}

	return response.json() as Promise<T>;
}

export async function fetchRaw(path: string, init?: RequestInit): Promise<TransportResponse> {
	const transport = getTransport();
	return transport.fetch(path, {
		method: (init?.method as TransportRequestInit['method']) ?? 'GET',
		headers: init?.headers as Record<string, string>,
		body: init?.body as string,
		signal: init?.signal ?? undefined
	});
}

export function subscribeSSE(path: string, signal?: AbortSignal): TransportEventSource {
	const transport = getTransport();
	return transport.subscribe(path, { signal });
}

export function resolveApiUrl(path: string): string {
	const transport = getTransport();
	return transport.resolveUrl(path);
}

export async function resolveApiUrlAsync(path: string): Promise<string> {
	const transport = getTransport();
	return transport.resolveUrlAsync(path);
}
