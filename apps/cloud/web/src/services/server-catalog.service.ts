import { writable, type Writable } from 'svelte/store';
import { signalingChatService } from '$services/signaling-chat.service';
import { playerService } from '$services/player.service';
import type {
	CatalogMovie,
	ServerCatalogMessage,
	ServerCatalogState
} from '$types/server-catalog.type';
import type { DataChannelEnvelope } from '$types/peer-library.type';

const initialState: ServerCatalogState = {
	movies: {}
};

class ServerCatalogService {
	public state: Writable<ServerCatalogState> = writable(initialState);

	private initialized = false;

	// Callback for server-side stream request handling
	public onStreamRequest: ((peerId: string, tmdbId: number) => void) | null = null;

	initialize(): void {
		if (this.initialized) return;
		this.initialized = true;

		signalingChatService.onServerCatalogMessage = (peerId, msg) => this.handleMessage(peerId, msg);
		signalingChatService.addPeerDisconnectedListener((peerId) =>
			this.handlePeerDisconnected(peerId)
		);
	}

	sendMovieCatalog(peerId: string, movies: CatalogMovie[]): void {
		const envelope: DataChannelEnvelope = {
			channel: 'server-catalog',
			payload: { type: 'catalog-movies', movies }
		};
		signalingChatService.sendToPeer(peerId, envelope);
	}

	requestCatalog(peerId: string): void {
		const envelope: DataChannelEnvelope = {
			channel: 'server-catalog',
			payload: { type: 'catalog-request' }
		};
		signalingChatService.sendToPeer(peerId, envelope);
	}

	requestStream(peerId: string, tmdbId: number): void {
		const envelope: DataChannelEnvelope = {
			channel: 'server-catalog',
			payload: { type: 'stream-request', tmdbId }
		};
		signalingChatService.sendToPeer(peerId, envelope);
	}

	sendStreamSession(peerId: string, sessionId: string, roomId: string, signalingUrl: string): void {
		const envelope: DataChannelEnvelope = {
			channel: 'server-catalog',
			payload: { type: 'stream-session', sessionId, roomId, signalingUrl }
		};
		signalingChatService.sendToPeer(peerId, envelope);
	}

	sendStreamError(peerId: string, error: string): void {
		const envelope: DataChannelEnvelope = {
			channel: 'server-catalog',
			payload: { type: 'stream-error', error }
		};
		signalingChatService.sendToPeer(peerId, envelope);
	}

	destroy(): void {
		this.initialized = false;
		this.onStreamRequest = null;
		this.state.set(initialState);
	}

	private handleMessage(peerId: string, msg: ServerCatalogMessage): void {
		switch (msg.type) {
			case 'catalog-start':
				this.state.update((s) => ({
					...s,
					movies: { ...s.movies, [peerId]: [] }
				}));
				break;

			case 'catalog-movies':
				this.state.update((s) => ({
					...s,
					movies: {
						...s.movies,
						[peerId]: [...(s.movies[peerId] ?? []), ...msg.movies]
					}
				}));
				break;

			case 'stream-request':
				this.onStreamRequest?.(peerId, msg.tmdbId);
				break;

			case 'stream-session':
				playerService.playRemote('Remote Stream', msg.sessionId, msg.roomId, msg.signalingUrl);
				break;

			case 'stream-error':
				console.error('[ServerCatalog] Stream error:', msg.error);
				break;
		}
	}

	private handlePeerDisconnected(peerId: string): void {
		this.state.update((s) => {
			const { [peerId]: _, ...rest } = s.movies;
			return { ...s, movies: rest };
		});
	}
}

export const serverCatalogService = new ServerCatalogService();
