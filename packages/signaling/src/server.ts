import { createServer, type IncomingMessage, type ServerResponse } from 'node:http';
import { WebSocketServer, WebSocket } from 'ws';
import { randomUUID } from 'node:crypto';
import { recoverMessageAddress } from 'viem';
import type {
	Room,
	ClientMessage,
	ServerMessage,
	AuthenticateMessage,
	StatusResponse
} from './types.js';

const AUTH_TIMEOUT_MS = 15_000;

export class SignalingServer {
	private rooms: Map<string, Room> = new Map();
	// peerId (lowercase Ethereum address) → set of room IDs
	private peerRooms: Map<string, Set<string>> = new Map();
	// peerId → active WebSocket (for evicting duplicate connections)
	private peerConnections: Map<string, WebSocket> = new Map();
	private port: number;

	constructor(port: number = 3002) {
		this.port = port;
	}

	start(): void {
		const httpServer = createServer((req: IncomingMessage, res: ServerResponse) => {
			this.handleHttp(req, res);
		});

		const wss = new WebSocketServer({ noServer: true });

		httpServer.on('upgrade', (req, socket, head) => {
			wss.handleUpgrade(req, socket, head, (ws) => {
				wss.emit('connection', ws, req);
			});
		});

		wss.on('connection', (ws: WebSocket) => {
			this.handleConnection(ws);
		});

		httpServer.listen(this.port, () => {
			console.log(`[signaling] Signaling server listening on port ${this.port}`);
		});
	}

	// ===== HTTP handler =====

	private handleHttp(req: IncomingMessage, res: ServerResponse): void {
		const url = req.url ?? '/';

		res.setHeader('Access-Control-Allow-Origin', '*');
		res.setHeader('Content-Type', 'application/json');

		if (url === '/health' && req.method === 'GET') {
			res.writeHead(200);
			res.end(JSON.stringify({ ok: true }));
			return;
		}

		if (url === '/status' && req.method === 'GET') {
			const status = this.getStatus();
			res.writeHead(200);
			res.end(JSON.stringify(status));
			return;
		}

		res.writeHead(404);
		res.end(JSON.stringify({ error: 'Not found' }));
	}

	private getStatus(): StatusResponse {
		const rooms = Array.from(this.rooms.values()).map((room) => ({
			id: room.id,
			peerCount: room.peers.size
		}));
		const totalPeers = rooms.reduce((sum, r) => sum + r.peerCount, 0);
		return { rooms, totalPeers };
	}

	// ===== WebSocket handler =====

	private handleConnection(ws: WebSocket): void {
		const nonce = randomUUID();

		// Send the challenge immediately on connect
		this.send(ws, { type: 'challenge', nonce });

		// Disconnect unauthenticated clients after timeout
		const authTimeout = setTimeout(() => {
			this.send(ws, { type: 'auth-failed', message: 'Authentication timeout' });
			ws.close();
		}, AUTH_TIMEOUT_MS);

		// The very first message must be `authenticate`
		ws.once('message', async (data) => {
			clearTimeout(authTimeout);

			let msg: AuthenticateMessage;
			try {
				const parsed = JSON.parse(data.toString()) as { type?: string; address?: string; signature?: string };
				if (parsed.type !== 'authenticate' || !parsed.address || !parsed.signature) {
					this.send(ws, { type: 'auth-failed', message: 'Expected authenticate message' });
					ws.close();
					return;
				}
				msg = parsed as AuthenticateMessage;
			} catch {
				this.send(ws, { type: 'auth-failed', message: 'Invalid message format' });
				ws.close();
				return;
			}

			// Verify the signature — the recovered address must match the claimed one
			let recovered: string;
			try {
				recovered = await recoverMessageAddress({
					message: nonce,
					signature: msg.signature as `0x${string}`
				});
			} catch {
				this.send(ws, { type: 'auth-failed', message: 'Signature recovery failed' });
				ws.close();
				return;
			}

			if (recovered.toLowerCase() !== msg.address.toLowerCase()) {
				this.send(ws, { type: 'auth-failed', message: 'Signature does not match address' });
				ws.close();
				return;
			}

			// Auth passed — use the verified address as the canonical peer ID
			const peerId = recovered.toLowerCase();

			// Evict any existing connection for this address
			const existing = this.peerConnections.get(peerId);
			if (existing && existing.readyState <= WebSocket.OPEN) {
				existing.close();
			}

			this.peerConnections.set(peerId, ws);
			this.peerRooms.set(peerId, new Set());

			this.send(ws, { type: 'connected', peer_id: peerId });

			// Register normal message and lifecycle handlers
			ws.on('message', (msgData) => {
				try {
					const clientMsg = JSON.parse(msgData.toString()) as ClientMessage;
					this.handleMessage(peerId, ws, clientMsg);
				} catch {
					this.send(ws, { type: 'error', message: 'Invalid message format' });
				}
			});

			ws.on('close', () => {
				this.peerConnections.delete(peerId);
				this.handleDisconnect(peerId);
			});

			ws.on('error', () => {
				this.peerConnections.delete(peerId);
				this.handleDisconnect(peerId);
			});
		});

		// If the connection drops before auth completes, clean up the timeout
		ws.once('close', () => clearTimeout(authTimeout));
	}

	private handleMessage(peerId: string, ws: WebSocket, msg: ClientMessage): void {
		switch (msg.type) {
			case 'join-room':
				this.handleJoinRoom(peerId, ws, msg.room_id);
				break;
			case 'leave-room':
				this.handleLeaveRoom(peerId, msg.room_id);
				break;
			case 'offer':
				this.relay(peerId, msg.room_id, msg.target_peer_id, {
					type: 'offer',
					room_id: msg.room_id,
					from_peer_id: peerId,
					sdp: msg.sdp
				});
				break;
			case 'answer':
				this.relay(peerId, msg.room_id, msg.target_peer_id, {
					type: 'answer',
					room_id: msg.room_id,
					from_peer_id: peerId,
					sdp: msg.sdp
				});
				break;
			case 'ice-candidate':
				this.relay(peerId, msg.room_id, msg.target_peer_id, {
					type: 'ice-candidate',
					room_id: msg.room_id,
					from_peer_id: peerId,
					candidate: msg.candidate,
					sdp_m_line_index: msg.sdp_m_line_index
				});
				break;
		}
	}

	private handleJoinRoom(peerId: string, ws: WebSocket, roomId: string): void {
		if (!this.rooms.has(roomId)) {
			this.rooms.set(roomId, { id: roomId, peers: new Map() });
		}

		const room = this.rooms.get(roomId)!;
		const existingPeers = Array.from(room.peers.keys());

		for (const [existingPeerId, existingWs] of room.peers) {
			if (existingPeerId !== peerId) {
				this.send(existingWs, { type: 'peer-joined', room_id: roomId, peer_id: peerId });
			}
		}

		room.peers.set(peerId, ws);
		this.peerRooms.get(peerId)?.add(roomId);

		this.send(ws, { type: 'room-peers', room_id: roomId, peers: existingPeers });
	}

	private handleLeaveRoom(peerId: string, roomId: string): void {
		const room = this.rooms.get(roomId);
		if (!room) return;

		room.peers.delete(peerId);
		this.peerRooms.get(peerId)?.delete(roomId);

		for (const remainingWs of room.peers.values()) {
			this.send(remainingWs, { type: 'peer-left', room_id: roomId, peer_id: peerId });
		}

		if (room.peers.size === 0) {
			this.rooms.delete(roomId);
		}
	}

	private handleDisconnect(peerId: string): void {
		const rooms = this.peerRooms.get(peerId) ?? new Set();
		for (const roomId of rooms) {
			this.handleLeaveRoom(peerId, roomId);
		}
		this.peerRooms.delete(peerId);
	}

	private relay(
		fromPeerId: string,
		roomId: string,
		targetPeerId: string,
		msg: ServerMessage
	): void {
		const room = this.rooms.get(roomId);
		if (!room) return;
		if (!room.peers.has(fromPeerId)) return;

		const targetWs = room.peers.get(targetPeerId);
		if (!targetWs) return;

		this.send(targetWs, msg);
	}

	private send(ws: WebSocket, msg: ServerMessage): void {
		if (ws.readyState === WebSocket.OPEN) {
			ws.send(JSON.stringify(msg));
		}
	}
}
