import type { ID } from '$types/core.type';

// ===== Download States =====

export type YouTubeDownloadState =
	| 'pending'
	| 'fetching'
	| 'downloading'
	| 'muxing'
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

// ===== Media Mode =====

export type MediaMode = 'audio' | 'video';

// ===== Download Mode =====

export type DownloadMode = 'audio' | 'video' | 'both';

export const DOWNLOAD_MODE_OPTIONS: { value: DownloadMode; label: string; description: string }[] =
	[
		{ value: 'both', label: 'Both', description: 'Download audio and video' },
		{ value: 'audio', label: 'Audio only', description: 'Download audio track only' },
		{ value: 'video', label: 'Video only', description: 'Download video with audio' }
	];

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
	videoOutputPath: string | null;
	audioOutputPath: string | null;
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

// ===== Stream URL Extraction =====

export interface YouTubeStreamFormat {
	itag: number;
	url: string;
	mimeType: string;
	bitrate: number;
	contentLength: number | null;
	width: number | null;
	height: number | null;
	qualityLabel: string | null;
	audioQuality: string | null;
	fps: number | null;
	isAudioOnly: boolean;
	isVideoOnly: boolean;
	codec: string;
	container: string;
}

export interface YouTubeStreamUrlResult {
	formats: YouTubeStreamFormat[];
	expiresAt: number;
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
	ytdlpVersion: string | null; // native-rust-{version}
}

export interface DownloaderStatus {
	/** Whether the native download engine is available */
	available: boolean;
	/** Download engine version if available */
	version: string | null;
	/** Whether the engine is initializing */
	downloading: boolean;
}

// ===== Service State =====

export interface YouTubeServiceState {
	initialized: boolean;
	loading: boolean;
	error: string | null;
	downloads: YouTubeDownloadProgress[];
	stats: YouTubeManagerStats | null;
	downloaderStatus: DownloaderStatus | null;
	// Current input state
	currentUrl: string;
	currentVideoInfo: YouTubeVideoInfo | null;
	currentPlaylistInfo: YouTubePlaylistInfo | null;
	fetchingInfo: boolean;
	fetchingVideoInfo: boolean;
	fetchingPlaylistInfo: boolean;
}

// ===== YouTube Content (downloaded media) =====

export interface YouTubeContent {
	youtubeId: string;
	title: string;
	thumbnailUrl: string | null;
	durationSeconds: number | null;
	channelName: string | null;
	channelId: string | null;
	hasVideo: boolean;
	hasAudio: boolean;
	videoSize: number | null;
	audioSize: number | null;
	isFavorite: boolean;
	favoritedAt: string | null;
	createdAt: string;
}

// ===== Settings (database) =====

export interface YouTubeSettings {
	id: ID;
	mediaMode: MediaMode;
	downloadMode: DownloadMode;
	defaultQuality: AudioQuality;
	defaultFormat: AudioFormat;
	defaultVideoQuality: VideoQuality;
	defaultVideoFormat: VideoFormat;
	poToken: string;
	cookies: string;
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
		case 'muxing':
			return 'info';
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
		case 'muxing':
			return 'Muxing';
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

// ===== Channel Feed (InnerTube Browse) =====

export interface YouTubeChannelFeedVideo {
	videoId: string;
	title: string;
	thumbnail: string;
	duration: number;
	durationText: string;
	views: number;
	viewsText: string;
	publishedText: string;
}

export interface YouTubeChannelFeedResponse {
	channelId: string;
	videos: YouTubeChannelFeedVideo[];
	continuation: string | null;
}

// ===== Channel Metadata =====

export interface YouTubeChannelMeta {
	channelId: string;
	avatar: string;
	description: string;
	subscriberText: string;
}

// ===== Right Panel =====

export interface RightPanelVideo {
	videoId: string;
	title: string;
	thumbnail: string;
	views?: number;
	viewsText?: string;
	publishedText?: string;
	uploaderName?: string;
	uploaderAvatar?: string;
	uploaderUrl?: string;
	uploaderVerified?: boolean;
	hasVideo?: boolean;
	hasAudio?: boolean;
}

// ===== Channel RSS Feed =====

export interface YouTubeRssVideo {
	videoId: string;
	title: string;
	published: string;
	publishedText: string;
	thumbnail: string;
	views: number;
	viewsText: string;
}

export interface YouTubeRssFeedResponse {
	channelId: string;
	channelName: string;
	videos: YouTubeRssVideo[];
}

// ===== oEmbed Metadata =====

export interface YouTubeOEmbedData {
	title: string;
	author_name: string;
	author_url: string;
	type: string;
	height: number;
	width: number;
	version: string;
	provider_name: string;
	provider_url: string;
	thumbnail_height: number;
	thumbnail_width: number;
	thumbnail_url: string;
	html: string;
}

/** Alias kept for backward compatibility with code that used the addon type name */
export type YouTubeOEmbedResponse = YouTubeOEmbedData;
