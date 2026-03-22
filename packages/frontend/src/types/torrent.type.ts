import type { ID } from 'frontend/types/core.type';

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
	addedAt: number; // unix timestamp
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

export interface AddTorrentRequest {
	source: string;
	downloadPath?: string;
	paused?: boolean;
}

export interface TorrentStatusResponse {
	initialized: boolean;
	download_path: string;
	stats: TorrentStats | null;
}

export interface TorrentConfigResponse {
	download_path: string;
}

// ===== Service State =====

export interface TorrentServiceState {
	initialized: boolean;
	loading: boolean;
	error: string | null;
	torrents: TorrentInfo[];
	allTorrents: TorrentInfo[];
	stats: TorrentStats | null;
	downloadPath: string;
	appName: string;
	appDownloadPath: string;
	libraryId: string;
}

// ===== Settings (localStorage) =====

export interface TorrentSettings {
	id: ID;
	downloadPath: string;
}

// ===== Helper Functions =====

export function formatBytes(bytes: number): string {
	if (bytes === 0) return '0 B';
	const units = ['B', 'KB', 'MB', 'GB', 'TB'];
	const i = Math.floor(Math.log(bytes) / Math.log(1024));
	const value = bytes / Math.pow(1024, i);
	return `${value.toFixed(i === 0 ? 0 : 1)} ${units[i]}`;
}

export function formatSpeed(bytesPerSec: number): string {
	if (bytesPerSec === 0) return '0 B/s';
	return `${formatBytes(bytesPerSec)}/s`;
}

export function formatEta(seconds: number | null): string {
	if (seconds === null || seconds <= 0) return '--';
	const hours = Math.floor(seconds / 3600);
	const minutes = Math.floor((seconds % 3600) / 60);
	const secs = Math.floor(seconds % 60);

	if (hours > 0) {
		return `${hours}h ${minutes}m`;
	}
	if (minutes > 0) {
		return `${minutes}m ${secs}s`;
	}
	return `${secs}s`;
}

export function getStateColor(state: TorrentState): string {
	switch (state) {
		case 'initializing':
			return 'info';
		case 'downloading':
			return 'primary';
		case 'seeding':
			return 'success';
		case 'paused':
			return 'warning';
		case 'checking':
			return 'info';
		case 'error':
			return 'error';
		default:
			return 'neutral';
	}
}

export function getStateLabel(state: TorrentState): string {
	switch (state) {
		case 'initializing':
			return 'Initializing';
		case 'downloading':
			return 'Downloading';
		case 'seeding':
			return 'Seeding';
		case 'paused':
			return 'Paused';
		case 'checking':
			return 'Checking';
		case 'error':
			return 'Error';
		default:
			return state;
	}
}
