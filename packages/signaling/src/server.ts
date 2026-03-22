import type * as Party from 'partykit/server';
import { recoverMessageAddress } from 'viem';
import type { ClientMessage, ServerMessage, PeerConnectionState } from './types.js';

const AUTH_TIMESTAMP_MAX_AGE_MS = 30_000;

export default class SignalingRoom implements Party.Server {
	readonly options: Party.ServerOptions = {
		hibernate: true
	};

	constructor(readonly room: Party.Room) {}

	// ===== Edge-level authentication =====

	static async onBeforeConnect(
		req: Party.Request,
		lobby: Party.Lobby
	): Promise<Party.Request | Response> {
		const url = new URL(req.url);
		const address = url.searchParams.get('address');
		const signature = url.searchParams.get('signature');
		const timestamp = url.searchParams.get('timestamp');

		if (!address || !signature || !timestamp) {
			return new Response('Missing auth parameters', { status: 401 });
		}

		const ts = parseInt(timestamp, 10);
		if (isNaN(ts) || Math.abs(Date.now() - ts) > AUTH_TIMESTAMP_MAX_AGE_MS) {
			return new Response('Expired or invalid timestamp', { status: 401 });
		}

		const roomId = lobby.id;
		const message = `partykit-auth:${roomId}:${timestamp}`;

		try {
			const recovered = await recoverMessageAddress({
				message,
				signature: signature as `0x${string}`
			});

			if (recovered.toLowerCase() !== address.toLowerCase()) {
				return new Response('Signature mismatch', { status: 401 });
			}

			req.headers.set('X-Peer-Id', recovered.toLowerCase());
			return req;
		} catch {
			return new Response('Signature verification failed', { status: 401 });
		}
	}

	// ===== Connection lifecycle =====

	async onConnect(
		connection: Party.Connection,
		ctx: Party.ConnectionContext
	): Promise<void> {
		const peerId = ctx.request.headers.get('X-Peer-Id');
		if (!peerId) {
			connection.close(4001, 'Unauthorized');
			return;
		}

		// Evict existing connections for this peer
		for (const conn of this.room.getConnections()) {
			const state = conn.state as PeerConnectionState | null;
			if (state?.peerId === peerId && conn.id !== connection.id) {
				conn.close(4002, 'Replaced by new connection');
			}
		}

		connection.setState({ peerId } satisfies PeerConnectionState);

		// Notify existing peers and collect their IDs
		const existingPeers: string[] = [];
		for (const conn of this.room.getConnections()) {
			const state = conn.state as PeerConnectionState | null;
			if (state?.peerId && conn.id !== connection.id) {
				existingPeers.push(state.peerId);
				this.send(conn, {
					type: 'peer-joined',
					room_id: this.room.id,
					peer_id: peerId
				});
			}
		}

		this.send(connection, { type: 'connected', peer_id: peerId });
		this.send(connection, {
			type: 'room-peers',
			room_id: this.room.id,
			peers: existingPeers
		});
	}

	async onMessage(message: string | ArrayBuffer, sender: Party.Connection): Promise<void> {
		const senderState = sender.state as PeerConnectionState | null;
		if (!senderState?.peerId) return;

		try {
			const msg = JSON.parse(message as string) as ClientMessage;
			this.handleMessage(senderState.peerId, msg);
		} catch {
			this.send(sender, { type: 'error', message: 'Invalid message format' });
		}
	}

	async onClose(connection: Party.Connection): Promise<void> {
		const state = connection.state as PeerConnectionState | null;
		if (!state?.peerId) return;

		for (const conn of this.room.getConnections()) {
			const connState = conn.state as PeerConnectionState | null;
			if (connState?.peerId && conn.id !== connection.id) {
				this.send(conn, {
					type: 'peer-left',
					room_id: this.room.id,
					peer_id: state.peerId
				});
			}
		}
	}

	// ===== HTTP endpoints =====

	async onRequest(req: Party.Request): Promise<Response> {
		if (req.method === 'GET') {
			const peers: string[] = [];
			for (const conn of this.room.getConnections()) {
				const state = conn.state as PeerConnectionState | null;
				if (state?.peerId) peers.push(state.peerId);
			}

			return new Response(
				JSON.stringify({
					room_id: this.room.id,
					peers,
					peerCount: peers.length
				}),
				{ headers: { 'Content-Type': 'application/json' } }
			);
		}

		return new Response('Not found', { status: 404 });
	}

	// ===== Internal helpers =====

	private handleMessage(fromPeerId: string, msg: ClientMessage): void {
		switch (msg.type) {
			case 'offer':
				this.relay(msg.target_peer_id, {
					type: 'offer',
					room_id: this.room.id,
					from_peer_id: fromPeerId,
					sdp: msg.sdp
				});
				break;
			case 'answer':
				this.relay(msg.target_peer_id, {
					type: 'answer',
					room_id: this.room.id,
					from_peer_id: fromPeerId,
					sdp: msg.sdp
				});
				break;
			case 'ice-candidate':
				this.relay(msg.target_peer_id, {
					type: 'ice-candidate',
					room_id: this.room.id,
					from_peer_id: fromPeerId,
					candidate: msg.candidate,
					sdp_m_line_index: msg.sdp_m_line_index,
					sdp_mid: msg.sdp_mid
				});
				break;
		}
	}

	private relay(targetPeerId: string, msg: ServerMessage): void {
		for (const conn of this.room.getConnections()) {
			const state = conn.state as PeerConnectionState | null;
			if (state?.peerId === targetPeerId) {
				this.send(conn, msg);
				return;
			}
		}
	}

	private send(conn: Party.Connection, msg: ServerMessage): void {
		conn.send(JSON.stringify(msg));
	}
}

SignalingRoom satisfies Party.Worker;
