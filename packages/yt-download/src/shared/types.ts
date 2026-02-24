// ===== Download States =====

export type DownloadState =
	| 'pending'
	| 'fetching'
	| 'downloading'
	| 'completed'
	| 'failed'
	| 'cancelled';

// ===== Audio Quality =====

export type AudioQuality = 'best' | 'high' | 'medium' | 'low';

// ===== Audio Format =====

export type AudioFormat = 'aac' | 'mp3' | 'opus';

// ===== Download Mode =====

export type DownloadMode = 'audio' | 'video';

// ===== Video Quality =====

export type VideoQuality = 'best' | '1080p' | '720p' | '480p';

// ===== Video Format =====

export type VideoFormat = 'mp4' | 'mkv' | 'webm';

// ===== API Request Types =====

export interface QueueDownloadRequest {
	url: string;
	videoId: string;
	title: string;
	mode: DownloadMode;
	quality: AudioQuality;
	format: AudioFormat;
	videoQuality?: VideoQuality;
	videoFormat?: VideoFormat;
}

export interface QueuePlaylistRequest {
	videos: { url: string; videoId: string; title: string }[];
	mode: DownloadMode;
	quality: AudioQuality;
	format: AudioFormat;
	videoQuality?: VideoQuality;
	videoFormat?: VideoFormat;
}

// ===== API Response Types =====

export interface DownloadProgress {
	downloadId: string;
	url: string;
	videoId: string;
	title: string;
	state: DownloadState;
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

export interface VideoInfo {
	title: string;
	duration: number; // seconds
	thumbnailUrl: string | null;
	uploader: string | null;
	videoId: string;
}

export interface PlaylistVideo {
	videoId: string;
	title: string;
	duration: number; // seconds
	thumbnailUrl: string | null;
	index: number; // Position in playlist (0-based)
}

export interface PlaylistInfo {
	playlistId: string;
	title: string;
	videoCount: number;
	videos: PlaylistVideo[];
	thumbnailUrl: string | null;
	author: string | null;
}

export interface ManagerStats {
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
	/** Whether download of yt-dlp binary is in progress */
	downloading: boolean;
}

export interface YtDownloadConfig {
	outputPath: string;
	defaultQuality: AudioQuality;
	defaultFormat: AudioFormat;
	poToken: string | null;
	cookies: string | null;
}

// ===== SSE Event Types =====

export type SSEEventType = 'progress' | 'stats' | 'connected';
