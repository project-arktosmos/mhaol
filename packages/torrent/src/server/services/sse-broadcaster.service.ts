import type { IncomingMessage, ServerResponse } from 'node:http';
import type { TorrentInfo, TorrentStats, SSEEventType } from '../../shared/types.js';

interface SSEClient {
	write: (data: string) => void;
	close: () => void;
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
				const encoder = new TextEncoder();
				const client: SSEClient = {
					write: (data) => controller.enqueue(encoder.encode(data)),
					close: () => controller.close(),
					closed: false
				};
				this.clients.add(client);

				const data = `event: connected\ndata: ${JSON.stringify({ message: 'Connected to torrent events' })}\n\n`;
				client.write(data);

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

	createNodeStream(req: IncomingMessage, res: ServerResponse): void {
		res.writeHead(200, {
			'Content-Type': 'text/event-stream',
			'Cache-Control': 'no-cache',
			Connection: 'keep-alive'
		});

		const client: SSEClient = {
			write: (data) => res.write(data),
			close: () => res.end(),
			closed: false
		};
		this.clients.add(client);

		const data = `event: connected\ndata: ${JSON.stringify({ message: 'Connected to torrent events' })}\n\n`;
		res.write(data);

		req.on('close', () => {
			client.closed = true;
			this.clients.delete(client);
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
		for (const client of this.clients) {
			if (client.closed) {
				this.clients.delete(client);
				continue;
			}
			try {
				client.write(message);
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
				client.close();
			} catch {
				// ignore
			}
		}
		this.clients.clear();
	}
}
