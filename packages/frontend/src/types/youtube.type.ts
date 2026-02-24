import type { ID } from '$types/core.type';

// ===== Download States =====

export type YouTubeDownloadState =
	| 'pending'
	| 'fetching'
	| 'downloading'
	| 'completed'
	| 'failed'
	| 'cancelled';

// ===== Audio Quality =====

export type AudioQuality = 'best' | 'high' | 'medium' | 'low';

export const AUDIO_QUALITY_OPTIONS: { value: AudioQuality; label: string; description: string }[] =
	[
		{ value: 'best', label: 'Best', description: 'Highest available quality' },
		{ value: 'high', label: 'High', description: '~192 kbps' },
		{ value: 'medium', label: 'Medium', description: '~128 kbps' },
		{ value: 'low', label: 'Low', description: '~96 kbps' }
	];

// ===== Audio Format =====

export type AudioFormat = 'aac' | 'mp3' | 'opus';

export const AUDIO_FORMAT_OPTIONS: { value: AudioFormat; label: string; extension: string }[] = [
	{ value: 'aac', label: 'AAC (.m4a)', extension: 'm4a' },
	{ value: 'mp3', label: 'MP3', extension: 'mp3' },
	{ value: 'opus', label: 'Opus', extension: 'opus' }
];

// ===== Download Mode =====

export type DownloadMode = 'audio' | 'video';

// ===== Video Quality =====

export type VideoQuality = 'best' | '1080p' | '720p' | '480p';

export const VIDEO_QUALITY_OPTIONS: { value: VideoQuality; label: string; description: string }[] =
	[
		{ value: 'best', label: 'Best', description: 'Highest available quality' },
		{ value: '1080p', label: '1080p', description: 'Full HD' },
		{ value: '720p', label: '720p', description: 'HD' },
		{ value: '480p', label: '480p', description: 'SD' }
	];

// ===== Video Format =====

export type VideoFormat = 'mp4' | 'mkv' | 'webm';

export const VIDEO_FORMAT_OPTIONS: { value: VideoFormat; label: string; extension: string }[] = [
	{ value: 'mp4', label: 'MP4', extension: 'mp4' },
	{ value: 'mkv', label: 'MKV', extension: 'mkv' },
	{ value: 'webm', label: 'WebM', extension: 'webm' }
];

// ===== API Response Types =====

export interface YouTubeDownloadProgress {
	downloadId: string;
	url: string;
	videoId: string;
	title: string;
	state: YouTubeDownloadState;
	progress: number; // 0.0 to 1.0
	downloadedBytes: number;
	totalBytes: number;
	outputPath: string | null;
	error: string | null;
	mode: DownloadMode;
	quality: AudioQuality;
	format: AudioFormat;
	videoQuality: VideoQuality | null;
	videoFormat: VideoFormat | null;
	thumbnailUrl: string | null;
	durationSeconds: number | null;
}

export interface YouTubeVideoInfo {
	title: string;
	duration: number; // seconds
	thumbnailUrl: string | null;
	uploader: string | null;
	videoId: string;
}

// ===== Playlist Types =====

export interface YouTubePlaylistVideo {
	videoId: string;
	title: string;
	duration: number; // seconds
	thumbnailUrl: string | null;
	index: number; // Position in playlist (0-based)
}

export interface YouTubePlaylistInfo {
	playlistId: string;
	title: string;
	videoCount: number;
	videos: YouTubePlaylistVideo[];
	thumbnailUrl: string | null;
	author: string | null;
}

export interface YouTubeManagerStats {
	activeDownloads: number;
	queuedDownloads: number;
	completedDownloads: number;
	failedDownloads: number;
	ytdlpAvailable: boolean;
	ytdlpVersion: string | null;
}

export interface YtDlpStatus {
	/** Whether yt-dlp binary is available */
	available: boolean;
	/** yt-dlp version if available */
	version: string | null;
	/** Whether download is in progress */
	downloading: boolean;
}

// ===== Service State =====

export interface YouTubeServiceState {
	initialized: boolean;
	loading: boolean;
	error: string | null;
	outputPath: string;
	downloads: YouTubeDownloadProgress[];
	stats: YouTubeManagerStats | null;
	ytdlpStatus: YtDlpStatus | null;
	// Current input state
	currentUrl: string;
	currentVideoInfo: YouTubeVideoInfo | null;
	currentPlaylistInfo: YouTubePlaylistInfo | null;
	fetchingInfo: boolean;
	fetchingVideoInfo: boolean;
	fetchingPlaylistInfo: boolean;
}

// ===== Settings (localStorage) =====

export interface YouTubeSettings {
	id: ID;
	downloadMode: DownloadMode;
	defaultQuality: AudioQuality;
	defaultFormat: AudioFormat;
	defaultVideoQuality: VideoQuality;
	defaultVideoFormat: VideoFormat;
	outputPath: string;
}

// ===== Authentication Config =====

export interface YouTubeConfig {
	/** YouTube Proof of Origin token to bypass bot detection */
	poToken: string | null;
	/** YouTube cookies from a logged-in session */
	cookies: string | null;
}

// ===== Helper Functions =====

export function formatDuration(seconds: number): string {
	const hours = Math.floor(seconds / 3600);
	const minutes = Math.floor((seconds % 3600) / 60);
	const secs = seconds % 60;

	if (hours > 0) {
		return `${hours}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
	}
	return `${minutes}:${secs.toString().padStart(2, '0')}`;
}

export function getStateColor(state: YouTubeDownloadState): string {
	switch (state) {
		case 'pending':
			return 'neutral';
		case 'fetching':
			return 'info';
		case 'downloading':
			return 'primary';
		case 'completed':
			return 'success';
		case 'failed':
			return 'error';
		case 'cancelled':
			return 'warning';
		default:
			return 'neutral';
	}
}

export function getStateLabel(state: YouTubeDownloadState): string {
	switch (state) {
		case 'pending':
			return 'Pending';
		case 'fetching':
			return 'Fetching Info';
		case 'downloading':
			return 'Downloading';
		case 'completed':
			return 'Completed';
		case 'failed':
			return 'Failed';
		case 'cancelled':
			return 'Cancelled';
		default:
			return state;
	}
}

/** Check if a URL is a YouTube playlist URL */
export function isPlaylistUrl(url: string): boolean {
	return url.includes('list=') || url.includes('/playlist');
}

/** Extract video ID from a YouTube URL */
export function extractVideoId(url: string): string | null {
	const patterns = [
		/(?:youtube\.com\/watch\?v=|youtu\.be\/|youtube\.com\/embed\/|youtube\.com\/v\/)([^&\n?#]+)/,
		/youtube\.com\/shorts\/([^&\n?#]+)/
	];

	for (const pattern of patterns) {
		const match = url.match(pattern);
		if (match) return match[1];
	}
	return null;
}

/** Extract playlist ID from a YouTube URL */
export function extractPlaylistId(url: string): string | null {
	const match = url.match(/[?&]list=([^&\n]+)/);
	return match ? match[1] : null;
}
