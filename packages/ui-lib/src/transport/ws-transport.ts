import type {
	Transport,
	TransportRequestInit,
	TransportResponse,
	TransportEventSource
} from './transport.type';
import type {
	RpcEnvelope,
	RpcMessage,
	RpcResponse,
	RpcChunk,
	RpcStreamEvent,
	RpcStreamEnd
} from './rpc.type';

const RPC_TIMEOUT_MS = 30_000;

interface PendingRequest {
	resolve: (res: TransportResponse) => void;
	reject: (err: Error) => void;
	chunks: string[];
	totalChunks: number;
	response: RpcResponse | null;
	timer: ReturnType<typeof setTimeout>;
}

export class WsTransport implements Transport {
	private pending = new Map<string, PendingRequest>();
	private streams = new Map<string, WsEventSource>();
	private ws: WebSocket;
	private serverUrl: string;

	constructor(ws: WebSocket, serverUrl: string) {
		this.ws = ws;
		this.serverUrl = serverUrl;

		this.ws.onmessage = (event) => {
			try {
				const envelope: RpcEnvelope = JSON.parse(event.data);
				if (envelope.channel === 'rpc') {
					this.handleMessage(envelope.payload);
				}
			} catch {
				// Ignore unparseable messages
			}
		};
	}

	private handleMessage(payload: RpcMessage): void {
		switch (payload.type) {
			case 'response':
				this.handleResponse(payload);
				break;
			case 'chunk':
				this.handleChunk(payload);
				break;
			case 'stream-event':
				this.handleStreamEvent(payload);
				break;
			case 'stream-end':
				this.handleStreamEnd(payload);
				break;
		}
	}

	async fetch(path: string, init?: TransportRequestInit): Promise<TransportResponse> {
		const id =
			typeof crypto.randomUUID === 'function'
				? crypto.randomUUID()
				: Math.random().toString(36).substring(2) + Date.now().toString(36);

		return new Promise<TransportResponse>((resolve, reject) => {
			const timer = setTimeout(() => {
				this.pending.delete(id);
				reject(new Error(`RPC timeout for ${path}`));
			}, RPC_TIMEOUT_MS);

			this.pending.set(id, {
				resolve,
				reject,
				chunks: [],
				totalChunks: 0,
				response: null,
				timer
			});

			const envelope: RpcEnvelope = {
				channel: 'rpc',
				payload: {
					id,
					type: 'request',
					method: init?.method ?? 'GET',
					path,
					headers: init?.headers,
					body: init?.body
				}
			};

			this.ws.send(JSON.stringify(envelope));
		});
	}

	subscribe(path: string, options?: { signal?: AbortSignal }): TransportEventSource {
		const id =
			typeof crypto.randomUUID === 'function'
				? crypto.randomUUID()
				: Math.random().toString(36).substring(2) + Date.now().toString(36);
		const source = new WsEventSource(() => {
			const envelope: RpcEnvelope = {
				channel: 'rpc',
				payload: { id, type: 'unsubscribe' }
			};
			this.ws.send(JSON.stringify(envelope));
			this.streams.delete(id);
		});
		this.streams.set(id, source);
		options?.signal?.addEventListener('abort', () => source.close());

		const envelope: RpcEnvelope = {
			channel: 'rpc',
			payload: { id, type: 'subscribe', path }
		};
		this.ws.send(JSON.stringify(envelope));

		return source;
	}

	resolveUrl(path: string): string {
		return `${this.serverUrl}${path}`;
	}

	async resolveUrlAsync(path: string): Promise<string> {
		return `${this.serverUrl}${path}`;
	}

	destroy(): void {
		for (const [id, req] of this.pending) {
			clearTimeout(req.timer);
			req.reject(new Error('Transport destroyed'));
			this.pending.delete(id);
		}
		for (const [id, stream] of this.streams) {
			stream.close();
			this.streams.delete(id);
		}
		this.ws.close();
	}

	private handleResponse(msg: RpcResponse): void {
		const req = this.pending.get(msg.id);
		if (!req) return;

		if (msg.chunked && msg.totalChunks) {
			req.response = msg;
			req.totalChunks = msg.totalChunks;
			return;
		}

		clearTimeout(req.timer);
		this.pending.delete(msg.id);

		const body = msg.body ?? '';
		req.resolve(this.buildResponse(msg.status, msg.statusText, msg.headers ?? {}, body));
	}

	private handleChunk(msg: RpcChunk): void {
		const req = this.pending.get(msg.id);
		if (!req) return;

		req.chunks[msg.seq] = msg.data;

		if (msg.final) {
			clearTimeout(req.timer);
			this.pending.delete(msg.id);

			const fullBody = req.chunks.join('');
			const resp = req.response!;
			req.resolve(this.buildResponse(resp.status, resp.statusText, resp.headers ?? {}, fullBody));
		}
	}

	private handleStreamEvent(msg: RpcStreamEvent): void {
		const stream = this.streams.get(msg.id);
		if (!stream) return;
		stream.emit(msg.eventType, msg.data);
	}

	private handleStreamEnd(msg: RpcStreamEnd): void {
		const stream = this.streams.get(msg.id);
		if (!stream) return;
		this.streams.delete(msg.id);
	}

	private buildResponse(
		status: number,
		statusText: string,
		headers: Record<string, string>,
		body: string
	): TransportResponse {
		return {
			ok: status >= 200 && status < 300,
			status,
			statusText,
			json: () => Promise.resolve(JSON.parse(body)),
			text: () => Promise.resolve(body),
			body: null,
			headers: { get: (name: string) => headers[name.toLowerCase()] ?? null }
		};
	}
}

class WsEventSource implements TransportEventSource {
	private listeners = new Map<string, Set<(data: string) => void>>();
	private closeFn: () => void;

	constructor(closeFn: () => void) {
		this.closeFn = closeFn;
	}

	addEventListener(type: string, callback: (data: string) => void): void {
		if (!this.listeners.has(type)) {
			this.listeners.set(type, new Set());
		}
		this.listeners.get(type)!.add(callback);
	}

	removeEventListener(type: string, callback: (data: string) => void): void {
		this.listeners.get(type)?.delete(callback);
	}

	emit(type: string, data: string): void {
		const callbacks = this.listeners.get(type);
		if (callbacks) {
			for (const cb of callbacks) {
				cb(data);
			}
		}
	}

	close(): void {
		this.closeFn();
		this.listeners.clear();
	}
}
