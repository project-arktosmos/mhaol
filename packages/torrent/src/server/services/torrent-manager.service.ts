import { existsSync, mkdirSync, rmSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import WebTorrent from 'webtorrent';
import type { Torrent } from 'webtorrent';
import type {
	TorrentInfo,
	TorrentStats,
	TorrentState,
	TorrentConfig,
	AddTorrentRequest
} from '../../shared/types.js';
import { parseMagnetUri } from '../utils/parse-magnet.js';
import { SSEBroadcasterService } from './sse-broadcaster.service.js';

const DEFAULT_TRACKERS = [
	'udp://tracker.opentrackr.org:1337/announce',
	'udp://open.stealth.si:80/announce',
	'udp://tracker.torrent.eu.org:451/announce',
	'udp://tracker.bittor.pw:1337/announce',
	'udp://public.popcorn-tracker.org:6969/announce',
	'udp://tracker.dler.org:6969/announce',
	'udp://exodus.desync.com:6969/announce',
	'udp://open.demonii.com:1337/announce'
];

interface TorrentMeta {
	addedAt: number;
	source: string;
}

type PersistenceCallback = (torrents: TorrentInfo[]) => void;

export class TorrentManagerService {
	private client: WebTorrent.Instance | null = null;
	private config: TorrentConfig = { downloadPath: '' };
	private meta: Map<string, TorrentMeta> = new Map();
	private broadcastInterval: ReturnType<typeof setInterval> | null = null;
	private persistenceCallback: PersistenceCallback | null = null;
	private _initialized = false;

	constructor(private broadcaster: SSEBroadcasterService) {}

	initialize(config: TorrentConfig): void {
		this.config = config;

		if (!existsSync(this.config.downloadPath)) {
			mkdirSync(this.config.downloadPath, { recursive: true });
		}

		// Disable WebRTC (wrtc: false) since we use UDP/HTTP trackers only
		// and node-datachannel requires native compilation
		this.client = new WebTorrent({
			dht: true,
			tracker: { wrtc: false },
			webSeeds: false
		} as ConstructorParameters<typeof WebTorrent>[0]);

		this.client.on('error', (err) => {
			const message = err instanceof Error ? err.message : String(err);
			console.error('[torrent] Client error:', message);
		});

		this._initialized = true;

		// Broadcast torrent list and stats every 1 second
		this.broadcastInterval = setInterval(() => {
			if (!this._initialized) return;
			const torrents = this.list();
			const stats = this.stats();
			this.broadcaster.broadcastTorrents(torrents);
			this.broadcaster.broadcastStats(stats);
			this.persistenceCallback?.(torrents);
		}, 1000);

		console.log(`[torrent] Initialized with download path: ${this.config.downloadPath}`);
	}

	isInitialized(): boolean {
		return this._initialized;
	}

	getConfig(): TorrentConfig {
		return { ...this.config };
	}

	updateConfig(updates: Partial<TorrentConfig>): void {
		Object.assign(this.config, updates);
		if (updates.downloadPath && !existsSync(updates.downloadPath)) {
			mkdirSync(updates.downloadPath, { recursive: true });
		}
	}

	setPersistenceCallback(cb: PersistenceCallback): void {
		this.persistenceCallback = cb;
	}

	// ===== Torrent Operations =====

	add(request: AddTorrentRequest): Promise<TorrentInfo> {
		return new Promise((resolve, reject) => {
			if (!this.client) {
				reject(new Error('Torrent client not initialized'));
				return;
			}

			const downloadPath = request.downloadPath || this.config.downloadPath;
			const trackers = [...DEFAULT_TRACKERS, ...(this.config.extraTrackers || [])];

			// Parse magnet URI for metadata
			let parsedMagnet: { infoHash: string; name: string } | null = null;
			if (request.source.startsWith('magnet:')) {
				parsedMagnet = parseMagnetUri(request.source);
			}

			const torrent = this.client.add(request.source, {
				path: downloadPath,
				announce: trackers
			});

			const addedAt = Math.floor(Date.now() / 1000);

			torrent.on('metadata', () => {
				this.meta.set(torrent.infoHash, {
					addedAt,
					source: request.source
				});
			});

			torrent.on('ready', () => {
				if (!this.meta.has(torrent.infoHash)) {
					this.meta.set(torrent.infoHash, {
						addedAt,
						source: request.source
					});
				}
				resolve(this.torrentToInfo(torrent));
			});

			torrent.on('error', (err) => {
				reject(err instanceof Error ? err : new Error(String(err)));
			});

			// If we have magnet metadata, store tracking info immediately
			if (parsedMagnet) {
				this.meta.set(parsedMagnet.infoHash, {
					addedAt,
					source: request.source
				});
			}
		});
	}

	list(): TorrentInfo[] {
		if (!this.client) return [];
		return this.client.torrents.map((t) => this.torrentToInfo(t));
	}

	pause(infoHash: string): void {
		const torrent = this.findTorrent(infoHash);
		if (!torrent) throw new Error(`Torrent not found: ${infoHash}`);
		torrent.pause();
	}

	resume(infoHash: string): void {
		const torrent = this.findTorrent(infoHash);
		if (!torrent) throw new Error(`Torrent not found: ${infoHash}`);
		torrent.resume();
	}

	remove(infoHash: string, deleteFiles = false): Promise<void> {
		return new Promise((resolve, reject) => {
			const torrent = this.findTorrent(infoHash);
			if (!torrent) {
				reject(new Error(`Torrent not found: ${infoHash}`));
				return;
			}

			torrent.destroy({ destroyStore: deleteFiles }, (err) => {
				if (err) {
					reject(err);
					return;
				}
				this.meta.delete(infoHash);
				resolve();
			});
		});
	}

	async removeAll(): Promise<number> {
		if (!this.client) return 0;
		const torrents = [...this.client.torrents];
		let count = 0;
		for (const torrent of torrents) {
			try {
				await this.remove(torrent.infoHash);
				count++;
			} catch {
				// continue removing others
			}
		}
		return count;
	}

	stats(): TorrentStats {
		if (!this.client) {
			return {
				totalDownloaded: 0,
				totalUploaded: 0,
				downloadSpeed: 0,
				uploadSpeed: 0,
				activeTorrents: 0
			};
		}

		let totalDownloaded = 0;
		let totalUploaded = 0;
		let activeTorrents = 0;

		for (const torrent of this.client.torrents) {
			totalDownloaded += torrent.downloaded;
			totalUploaded += torrent.uploaded;
			if (!torrent.done && !torrent.paused) {
				activeTorrents++;
			}
		}

		return {
			totalDownloaded,
			totalUploaded,
			downloadSpeed: this.client.downloadSpeed,
			uploadSpeed: this.client.uploadSpeed,
			activeTorrents
		};
	}

	debugInfo(): string[] {
		const lines: string[] = [];
		lines.push(`Initialized: ${this._initialized}`);
		lines.push(`Download path: ${this.config.downloadPath}`);

		if (!this.client) {
			lines.push('Client: not initialized');
			return lines;
		}

		lines.push(`Torrents: ${this.client.torrents.length}`);
		lines.push(`Download speed: ${this.client.downloadSpeed} B/s`);
		lines.push(`Upload speed: ${this.client.uploadSpeed} B/s`);

		for (const torrent of this.client.torrents) {
			lines.push(`--- ${torrent.name} ---`);
			lines.push(`  Info hash: ${torrent.infoHash}`);
			lines.push(`  Progress: ${(torrent.progress * 100).toFixed(1)}%`);
			lines.push(`  Size: ${torrent.length}`);
			lines.push(`  Peers: ${torrent.numPeers}`);
			lines.push(`  Done: ${torrent.done}`);
			lines.push(`  Paused: ${torrent.paused}`);
			lines.push(`  DL speed: ${torrent.downloadSpeed} B/s`);
			lines.push(`  UL speed: ${torrent.uploadSpeed} B/s`);
		}

		return lines;
	}

	clearStorage(): void {
		// Remove all torrents first
		if (this.client) {
			for (const torrent of [...this.client.torrents]) {
				try {
					torrent.destroy({ destroyStore: true });
				} catch {
					// ignore
				}
			}
		}
		this.meta.clear();

		// Clear download directory contents
		if (existsSync(this.config.downloadPath)) {
			try {
				const entries = readdirSync(this.config.downloadPath);
				for (const entry of entries) {
					const fullPath = join(this.config.downloadPath, entry);
					rmSync(fullPath, { recursive: true, force: true });
				}
			} catch {
				// ignore cleanup errors
			}
		}
	}

	destroy(): void {
		if (this.broadcastInterval) {
			clearInterval(this.broadcastInterval);
			this.broadcastInterval = null;
		}
		if (this.client) {
			this.client.destroy();
			this.client = null;
		}
		this._initialized = false;
	}

	// ===== Internal Helpers =====

	private findTorrent(infoHash: string): Torrent | undefined {
		return this.client?.torrents.find((t) => t.infoHash === infoHash);
	}

	private torrentToInfo(torrent: Torrent): TorrentInfo {
		const meta = this.meta.get(torrent.infoHash);
		const state = this.deriveTorrentState(torrent);

		// WebTorrent doesn't distinguish seeds from peers in its public API
		// Use numPeers for both; seeds count is not reliably available
		const seeds = 0;

		return {
			infoHash: torrent.infoHash,
			name: torrent.name || `Torrent ${torrent.infoHash.slice(0, 8)}`,
			size: torrent.length || 0,
			progress: torrent.progress,
			downloadSpeed: torrent.downloadSpeed,
			uploadSpeed: torrent.uploadSpeed,
			peers: torrent.numPeers,
			seeds,
			state,
			addedAt: meta?.addedAt ?? Math.floor(Date.now() / 1000),
			eta: torrent.timeRemaining === Infinity ? null : Math.ceil(torrent.timeRemaining / 1000),
			outputPath: torrent.path || null
		};
	}

	private deriveTorrentState(torrent: Torrent): TorrentState {
		if (torrent.paused) return 'paused';
		if (torrent.done) return 'seeding';
		if (torrent.progress > 0) return 'downloading';
		// If metadata is available (name exists and length > 0), it's downloading
		if (torrent.name && torrent.length > 0) return 'downloading';
		return 'initializing';
	}
}
