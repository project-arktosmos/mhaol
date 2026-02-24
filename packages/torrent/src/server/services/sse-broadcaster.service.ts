import type { TorrentInfo, TorrentStats, SSEEventType } from '../../shared/types.js';

interface SSEClient {
	controller: ReadableStreamDefaultController;
	closed: boolean;
}

export class SSEBroadcasterService {
	private clients: Set<SSEClient> = new Set();
	private heartbeatInterval: ReturnType<typeof setInterval> | null = null;

	constructor() {
		this.heartbeatInterval = setInterval(() => {
			this.sendRaw(':\n\n');
		}, 30_000);
	}

	createStream(request: Request): Response {
		const stream = new ReadableStream({
			start: (controller) => {
				const client: SSEClient = { controller, closed: false };
				this.clients.add(client);

				const data = `event: connected\ndata: ${JSON.stringify({ message: 'Connected to torrent events' })}\n\n`;
				controller.enqueue(new TextEncoder().encode(data));

				request.signal.addEventListener('abort', () => {
					client.closed = true;
					this.clients.delete(client);
				});
			},
			cancel: () => {
				// Stream cancelled by client
			}
		});

		return new Response(stream, {
			headers: {
				'Content-Type': 'text/event-stream',
				'Cache-Control': 'no-cache',
				Connection: 'keep-alive'
			}
		});
	}

	broadcastTorrents(torrents: TorrentInfo[]): void {
		this.send('torrents', torrents);
	}

	broadcastStats(stats: TorrentStats): void {
		this.send('stats', stats);
	}

	private send(event: SSEEventType, data: unknown): void {
		const message = `event: ${event}\ndata: ${JSON.stringify(data)}\n\n`;
		this.sendRaw(message);
	}

	private sendRaw(message: string): void {
		const encoded = new TextEncoder().encode(message);
		for (const client of this.clients) {
			if (client.closed) {
				this.clients.delete(client);
				continue;
			}
			try {
				client.controller.enqueue(encoded);
			} catch {
				client.closed = true;
				this.clients.delete(client);
			}
		}
	}

	destroy(): void {
		if (this.heartbeatInterval) {
			clearInterval(this.heartbeatInterval);
			this.heartbeatInterval = null;
		}
		for (const client of this.clients) {
			try {
				client.controller.close();
			} catch {
				// ignore
			}
		}
		this.clients.clear();
	}
}
