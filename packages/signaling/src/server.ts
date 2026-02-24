import { createServer, type IncomingMessage, type ServerResponse } from 'node:http';
import { WebSocketServer, WebSocket } from 'ws';
import { randomUUID } from 'node:crypto';
import type {
	Room,
	ClientMessage,
	ServerMessage,
	StatusResponse
} from './types.js';

export class SignalingServer {
	private rooms: Map<string, Room> = new Map();
	private peerRooms: Map<string, Set<string>> = new Map();
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
		const peerId = randomUUID();
		this.peerRooms.set(peerId, new Set());

		this.send(ws, { type: 'connected', peer_id: peerId });

		ws.on('message', (data) => {
			try {
				const msg = JSON.parse(data.toString()) as ClientMessage;
				this.handleMessage(peerId, ws, msg);
			} catch {
				this.send(ws, { type: 'error', message: 'Invalid message format' });
			}
		});

		ws.on('close', () => {
			this.handleDisconnect(peerId);
		});

		ws.on('error', () => {
			this.handleDisconnect(peerId);
		});
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

		// Notify existing peers that a new peer joined
		for (const [existingPeerId, existingWs] of room.peers) {
			if (existingPeerId !== peerId) {
				this.send(existingWs, { type: 'peer-joined', room_id: roomId, peer_id: peerId });
			}
		}

		// Add peer to room
		room.peers.set(peerId, ws);
		this.peerRooms.get(peerId)?.add(roomId);

		// Send the current peers list to the joining peer
		this.send(ws, { type: 'room-peers', room_id: roomId, peers: existingPeers });
	}

	private handleLeaveRoom(peerId: string, roomId: string): void {
		const room = this.rooms.get(roomId);
		if (!room) return;

		room.peers.delete(peerId);
		this.peerRooms.get(peerId)?.delete(roomId);

		// Notify remaining peers
		for (const remainingWs of room.peers.values()) {
			this.send(remainingWs, { type: 'peer-left', room_id: roomId, peer_id: peerId });
		}

		// Clean up empty rooms
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
