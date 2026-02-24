import type { DownloadProgress, ManagerStats, SSEEventType } from '../../shared/types.js';

interface SSEClient {
	controller: ReadableStreamDefaultController;
	closed: boolean;
}

export class SSEBroadcasterService {
	private clients: Set<SSEClient> = new Set();
	private heartbeatInterval: ReturnType<typeof setInterval> | null = null;

	constructor() {
		// Send heartbeat every 30s to keep connections alive
		this.heartbeatInterval = setInterval(() => {
			this.sendRaw(':\n\n');
		}, 30_000);
	}

	/** Create an SSE response for a route handler */
	createStream(request: Request): Response {
		const stream = new ReadableStream({
			start: (controller) => {
				const client: SSEClient = { controller, closed: false };
				this.clients.add(client);

				// Send initial connected event
				const data = `event: connected\ndata: ${JSON.stringify({ message: 'Connected to download events' })}\n\n`;
				controller.enqueue(new TextEncoder().encode(data));

				// Cleanup on close
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

	/** Broadcast a download progress update to all clients */
	broadcastProgress(progress: DownloadProgress): void {
		this.send('progress', progress);
	}

	/** Broadcast updated stats to all clients */
	broadcastStats(stats: ManagerStats): void {
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
