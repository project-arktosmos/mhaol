import { createHelia, type Helia } from 'helia';
import { unixfs, type UnixFS } from '@helia/unixfs';
import { createLibp2p } from 'libp2p';
import { webSockets } from '@libp2p/websockets';
import { webTransport } from '@libp2p/webtransport';
import { noise } from '@chainsafe/libp2p-noise';
import { yamux } from '@chainsafe/libp2p-yamux';
import { identify } from '@libp2p/identify';
import { bootstrap } from '@libp2p/bootstrap';
import { preSharedKey } from '@libp2p/pnet';
import { multiaddr } from '@multiformats/multiaddr';
import { CID } from 'multiformats/cid';
import { concat as uint8Concat } from 'uint8arrays/concat';
import { toString as uint8ToString } from 'uint8arrays/to-string';
import { IDBBlockstore } from 'blockstore-idb';
import { IDBDatastore } from 'datastore-idb';

/**
 * Configuration for the in-browser IPFS node. Defaults are wired to the
 * cloud rendezvous when running locally — change the bootstrap multiaddr
 * to dial a remote rendezvous, and change the swarm key to match the one
 * configured on the cloud / rendezvous side.
 *
 * Notes on the connectivity model:
 *  - Browsers can't speak raw TCP, so we dial WebSocket / WebTransport
 *    multiaddrs only. The rendezvous node has to expose a `/ws` listener
 *    (apps/rendezvous) for this to actually connect.
 *  - The swarm key (`/key/swarm/psk/1.0.0/...`) is the same private-swarm
 *    PSK that the cloud and rendezvous read from `<DATA_DIR>/swarm.key`.
 *    Without it the libp2p `pnet` handshake rejects the connection.
 */
export interface PlayerIpfsConfig {
	/**
	 * One or more bootstrap multiaddrs. Each must include the peer id and
	 * use a browser-dialable transport (`/ws`, `/wss`, or
	 * `/webtransport`). Example:
	 *   `/ip4/127.0.0.1/tcp/14002/ws/p2p/12D3...`
	 */
	bootstrapMultiaddrs: string[];
	/**
	 * Full swarm-key file contents (the literal text starting with
	 * `/key/swarm/psk/1.0.0/`). Mandatory — the swarm is private.
	 */
	swarmKey: string;
}

let cached: Promise<PlayerIpfsClient> | null = null;
let cachedConfigKey: string | null = null;

export interface PlayerIpfsClient {
	helia: Helia;
	fs: UnixFS;
	stop(): Promise<void>;
	peerCount(): number;
}

function configKey(config: PlayerIpfsConfig): string {
	return JSON.stringify({
		boot: [...config.bootstrapMultiaddrs].sort(),
		key: config.swarmKey
	});
}

export async function getPlayerIpfsClient(config: PlayerIpfsConfig): Promise<PlayerIpfsClient> {
	const key = configKey(config);
	if (cached && cachedConfigKey === key) return cached;
	if (cached && cachedConfigKey !== key) {
		const old = cached;
		cached = null;
		void old.then((c) => c.stop()).catch(() => {});
	}

	cached = createPlayerIpfsClient(config);
	cachedConfigKey = key;
	return cached;
}

async function createPlayerIpfsClient(config: PlayerIpfsConfig): Promise<PlayerIpfsClient> {
	if (config.bootstrapMultiaddrs.length === 0) {
		throw new Error('Bootstrap multiaddrs are required to dial the rendezvous');
	}
	if (!config.swarmKey || !config.swarmKey.startsWith('/key/swarm/psk/1.0.0/')) {
		throw new Error('Swarm key must start with /key/swarm/psk/1.0.0/');
	}

	const swarmKeyBytes = new TextEncoder().encode(config.swarmKey);

	const blockstore = new IDBBlockstore('mhaol-player-blocks');
	await blockstore.open();
	const datastore = new IDBDatastore('mhaol-player-data');
	await datastore.open();

	const libp2p = await createLibp2p({
		datastore,
		// Browsers can't listen — they only dial.
		addresses: { listen: [] },
		transports: [
			webSockets({
				// Allow ws:// (cleartext) for localhost dev so the player can
				// dial a non-TLS rendezvous on the same machine. The pnet
				// handshake on top still gates membership.
				filter: (addrs) => addrs
			}),
			webTransport()
		],
		connectionEncrypters: [noise()],
		streamMuxers: [yamux()],
		connectionProtector: preSharedKey({ psk: swarmKeyBytes }),
		peerDiscovery: [
			bootstrap({
				list: config.bootstrapMultiaddrs
			})
		],
		services: {
			identify: identify()
		},
		connectionGater: {
			// The rendezvous bootstrap addrs are always trusted.
			denyDialMultiaddr: () => false
		}
	});

	const helia = await createHelia({
		blockstore,
		datastore,
		libp2p,
		// Don't run the public bootstrap list — we're on a private swarm.
		start: true
	});

	// Best-effort dial each bootstrap so the connection is ready before
	// the first UnixFS read.
	for (const addr of config.bootstrapMultiaddrs) {
		try {
			await helia.libp2p.dial(multiaddr(addr));
		} catch (err) {
			console.warn('[player-ipfs] bootstrap dial failed', addr, err);
		}
	}

	return {
		helia,
		fs: unixfs(helia),
		async stop() {
			try {
				await helia.stop();
			} catch (err) {
				console.warn('[player-ipfs] helia stop failed', err);
			}
		},
		peerCount() {
			return helia.libp2p.getConnections().length;
		}
	};
}

export interface ReadFileOptions {
	signal?: AbortSignal;
	onProgress?: (bytesSoFar: number) => void;
}

/**
 * Read a UnixFS file at the given CID into a single `Uint8Array`. The
 * caller is responsible for chunking / streaming into a `<video>` element
 * when the file is large; this helper is appropriate for small things
 * like firkin JSON bodies.
 */
export async function catBytes(
	client: PlayerIpfsClient,
	cidStr: string,
	options: ReadFileOptions = {}
): Promise<Uint8Array> {
	const cid = CID.parse(cidStr);
	const chunks: Uint8Array[] = [];
	let total = 0;
	for await (const chunk of client.fs.cat(cid, { signal: options.signal })) {
		chunks.push(chunk);
		total += chunk.byteLength;
		options.onProgress?.(total);
	}
	return uint8Concat(chunks);
}

/** Convenience helper — fetch + UTF-8 decode for JSON-like payloads. */
export async function catText(
	client: PlayerIpfsClient,
	cidStr: string,
	options?: ReadFileOptions
): Promise<string> {
	const bytes = await catBytes(client, cidStr, options);
	return uint8ToString(bytes, 'utf8');
}

/**
 * Same as `catBytes` but materialises the result as a `Blob` typed with
 * `mimeType` so it can be assigned to a `<video>` `src` directly.
 */
export async function catAsBlob(
	client: PlayerIpfsClient,
	cidStr: string,
	mimeType: string,
	options?: ReadFileOptions
): Promise<Blob> {
	const bytes = await catBytes(client, cidStr, options);
	// Re-wrap into a fresh ArrayBuffer-backed view so the Blob constructor
	// accepts it under the strict `BufferSource` typing (Helia returns
	// `Uint8Array<ArrayBufferLike>`, which TypeScript rejects directly).
	const view = new Uint8Array(bytes.byteLength);
	view.set(bytes);
	return new Blob([view], { type: mimeType });
}
