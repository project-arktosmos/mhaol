import type * as Party from 'partykit/server';
import { recoverMessageAddress, getAddress, isAddress } from 'viem';
import type { ClientMessage, ServerMessage, PeerConnectionState, IceServerConfig } from './types.js';

const AUTH_TIMESTAMP_MAX_AGE_MS = 30_000;
const HANDSHAKES_ROOM = 'handshakes';

function isEip55Room(roomId: string): boolean {
	if (!isAddress(roomId)) return false;
	try {
		return getAddress(roomId) === roomId;
	} catch {
		return false;
	}
}
const ICE_CACHE_TTL_MS = 12 * 60 * 60 * 1000; // 12 hours

export default class SignalingRoom implements Party.Server {
	readonly options: Party.ServerOptions = {
		hibernate: true
	};

	private cachedIceServers: IceServerConfig[] | null = null;
	private iceCacheExpiry = 0;

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
		const passportRaw = url.searchParams.get('passport_raw');
		const passportSignature = url.searchParams.get('passport_signature');

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
			// Verify the auth challenge signature
			const recovered = await recoverMessageAddress({
				message,
				signature: signature as `0x${string}`
			});

			if (recovered.toLowerCase() !== address.toLowerCase()) {
				return new Response('Signature mismatch', { status: 401 });
			}

			req.headers.set('X-Peer-Id', recovered.toLowerCase());

			// Verify passport if provided
			let name = '';
			let instanceType = '';

			if (passportRaw && passportSignature) {
				const passportRecovered = await recoverMessageAddress({
					message: passportRaw,
					signature: passportSignature as `0x${string}`
				});

				if (passportRecovered.toLowerCase() !== address.toLowerCase()) {
					return new Response('Passport signature mismatch', { status: 401 });
				}

				const payload = JSON.parse(passportRaw);
				name = payload.name ?? '';
				instanceType = payload.instanceType ?? '';
			}

			// Room ACL enforcement
			if (roomId === HANDSHAKES_ROOM) {
				// Open room — any authenticated user can join
			} else if (isEip55Room(roomId)) {
				const peerChecksummed = getAddress(address as `0x${string}`);
				const isOwner = peerChecksummed === roomId;

				if (isOwner) {
					// Only server-type passports can own rooms
					if (instanceType !== 'server') {
						return new Response('Only server instances can create rooms', { status: 403 });
					}
				} else {
					// Non-owner must present a valid endorsement from the room owner
					const endorserSignature = url.searchParams.get('endorser_signature');
					if (!endorserSignature || !passportRaw) {
						return new Response('Endorsement required for this room', { status: 403 });
					}

					// Verify the endorser signed the connecting client's passport
					const endorserRecovered = await recoverMessageAddress({
						message: passportRaw,
						signature: endorserSignature as `0x${string}`
					});

					if (getAddress(endorserRecovered) !== roomId) {
						return new Response('Invalid endorsement for this room', { status: 403 });
					}
				}
			} else {
				return new Response('Invalid room name', { status: 403 });
			}

			req.headers.set('X-Peer-Name', name);
			req.headers.set('X-Peer-Instance-Type', instanceType);
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

		const name = ctx.request.headers.get('X-Peer-Name') ?? '';
		const instanceType = ctx.request.headers.get('X-Peer-Instance-Type') ?? '';

		// Evict existing connections for this peer
		for (const conn of this.room.getConnections()) {
			const state = conn.state as PeerConnectionState | null;
			if (state?.peerId === peerId && conn.id !== connection.id) {
				conn.close(4002, 'Replaced by new connection');
			}
		}

		connection.setState({ peerId, name, instanceType } satisfies PeerConnectionState);

		// Notify existing peers and collect their info
		const existingPeers: { peer_id: string; name: string; instance_type: string }[] = [];
		for (const conn of this.room.getConnections()) {
			const state = conn.state as PeerConnectionState | null;
			if (state?.peerId && conn.id !== connection.id) {
				existingPeers.push({
					peer_id: state.peerId,
					name: state.name,
					instance_type: state.instanceType
				});
				this.send(conn, {
					type: 'peer-joined',
					room_id: this.room.id,
					peer_id: peerId,
					name,
					instance_type: instanceType
				});
			}
		}

		const iceServers = await this.getIceServers();
		this.send(connection, {
			type: 'connected',
			peer_id: peerId,
			name,
			instance_type: instanceType,
			ice_servers: iceServers
		});
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
		const corsHeaders: Record<string, string> = {
			'Access-Control-Allow-Origin': '*',
			'Access-Control-Allow-Methods': 'GET, OPTIONS',
			'Access-Control-Allow-Headers': 'Content-Type'
		};

		if (req.method === 'OPTIONS') {
			return new Response(null, { status: 204, headers: corsHeaders });
		}

		if (req.method === 'GET') {
			const url = new URL(req.url);

			// Temporary debug endpoint to verify env vars reach the deployed worker
			if (url.pathname.endsWith('/debug-env')) {
				const domain = this.room.env.METERED_DOMAIN as string | undefined;
				const hasKey = !!(this.room.env.METERED_SECRET_KEY as string | undefined);
				return new Response(
					JSON.stringify({
						metered_domain: domain ?? null,
						has_secret_key: hasKey,
						cached_ice_servers: this.cachedIceServers?.length ?? null,
						cache_expires_in_ms: this.iceCacheExpiry > 0 ? this.iceCacheExpiry - Date.now() : null
					}),
					{ headers: { 'Content-Type': 'application/json', ...corsHeaders } }
				);
			}

			const peers: { peer_id: string; name: string; instance_type: string }[] = [];
			for (const conn of this.room.getConnections()) {
				const state = conn.state as PeerConnectionState | null;
				if (state?.peerId) {
					peers.push({
						peer_id: state.peerId,
						name: state.name,
						instance_type: state.instanceType
					});
				}
			}

			return new Response(
				JSON.stringify({
					room_id: this.room.id,
					peers,
					peerCount: peers.length
				}),
				{ headers: { 'Content-Type': 'application/json', ...corsHeaders } }
			);
		}

		return new Response('Not found', { status: 404, headers: corsHeaders });
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

	// ===== TURN credential distribution =====

	private async getIceServers(): Promise<IceServerConfig[]> {
		if (this.cachedIceServers && Date.now() < this.iceCacheExpiry) {
			return this.cachedIceServers;
		}

		const domain = this.room.env.METERED_DOMAIN as string | undefined;
		const secretKey = this.room.env.METERED_SECRET_KEY as string | undefined;
		if (!domain || !secretKey) {
			console.warn('[signaling] METERED_DOMAIN or METERED_SECRET_KEY not set, no TURN servers available');
			return [];
		}

		try {
			const res = await fetch(
				`https://${domain}/api/v1/turn/credentials?apiKey=${secretKey}`
			);
			if (!res.ok) {
				console.warn(`[signaling] Metered API returned ${res.status}, using cached ICE servers`);
				return this.cachedIceServers ?? [];
			}

			const servers: IceServerConfig[] = await res.json();
			const turnCount = servers.filter(s => {
				const urls = Array.isArray(s.urls) ? s.urls : [s.urls];
				return urls.some(u => u.startsWith('turn:') || u.startsWith('turns:'));
			}).length;
			console.log(`[signaling] Fetched ${servers.length} ICE servers (${turnCount} TURN)`);
			this.cachedIceServers = servers;
			this.iceCacheExpiry = Date.now() + ICE_CACHE_TTL_MS;
			return servers;
		} catch (err) {
			console.error('[signaling] Metered API fetch error:', err);
			return this.cachedIceServers ?? [];
		}
	}
}

SignalingRoom satisfies Party.Worker;
