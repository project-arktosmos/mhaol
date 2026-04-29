export type TransportMethod = 'GET' | 'POST' | 'PUT' | 'DELETE';

export interface TransportRequestInit {
	method?: TransportMethod;
	headers?: Record<string, string>;
	body?: string;
	signal?: AbortSignal;
}

export interface TransportResponse {
	ok: boolean;
	status: number;
	statusText: string;
	json(): Promise<any>;
	text(): Promise<string>;
	body: ReadableStream<Uint8Array> | null;
	headers: { get(name: string): string | null };
}

export interface TransportEventSource {
	addEventListener(type: string, callback: (data: string) => void): void;
	removeEventListener(type: string, callback: (data: string) => void): void;
	close(): void;
}

export interface Transport {
	fetch(path: string, init?: TransportRequestInit): Promise<TransportResponse>;
	subscribe(path: string, options?: { signal?: AbortSignal }): TransportEventSource;
	resolveUrl(path: string): string;
	resolveUrlAsync(path: string): Promise<string>;
}
