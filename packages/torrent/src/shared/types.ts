// ===== Torrent States =====

export type TorrentState =
	| 'initializing'
	| 'downloading'
	| 'seeding'
	| 'paused'
	| 'checking'
	| 'error';

// ===== API Response Types =====

export interface TorrentInfo {
	infoHash: string;
	name: string;
	size: number;
	progress: number; // 0.0 to 1.0
	downloadSpeed: number; // bytes/sec
	uploadSpeed: number; // bytes/sec
	peers: number;
	seeds: number;
	state: TorrentState;
	addedAt: number; // unix timestamp (seconds)
	eta: number | null; // seconds remaining
	outputPath: string | null;
}

export interface TorrentStats {
	totalDownloaded: number;
	totalUploaded: number;
	downloadSpeed: number; // bytes/sec
	uploadSpeed: number; // bytes/sec
	activeTorrents: number;
}

// ===== API Request Types =====

export interface AddTorrentRequest {
	source: string; // magnet URI, HTTP URL, or file path
	downloadPath?: string;
	paused?: boolean;
}

// ===== Configuration =====

export interface TorrentConfig {
	downloadPath: string;
	extraTrackers?: string[];
}

// ===== SSE Event Types =====

export type SSEEventType = 'torrents' | 'stats' | 'connected';
