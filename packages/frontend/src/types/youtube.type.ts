import type { ID } from '$types/core.type';

// Re-export all API types from the youtube addon
export type {
	YouTubeDownloadState,
	AudioQuality,
	AudioFormat,
	MediaMode,
	DownloadMode,
	VideoQuality,
	VideoFormat,
	SubtitleMode,
	SubtitleTrack,
	DownloadedSubtitle,
	YouTubeDownloadProgress,
	YouTubeVideoInfo,
	YouTubeStreamFormat,
	YouTubeStreamUrlResult,
	YouTubePlaylistVideo,
	YouTubePlaylistInfo,
	YouTubeManagerStats,
	DownloaderStatus,
	YouTubeContent,
	YouTubeChannelFeedVideo,
	YouTubeChannelFeedResponse,
	YouTubeChannelMeta,
	RightPanelVideo,
	YouTubeRssVideo,
	YouTubeRssFeedResponse,
	YouTubeOEmbedData,
	YouTubeOEmbedResponse
} from 'addons/youtube/types';

export {
	formatDuration,
	isPlaylistUrl,
	extractVideoId,
	extractPlaylistId
} from 'addons/youtube/types';

import type {
	YouTubeDownloadState,
	AudioQuality,
	AudioFormat,
	DownloadMode,
	VideoQuality,
	VideoFormat,
	SubtitleMode,
	YouTubeDownloadProgress,
	YouTubeVideoInfo,
	YouTubePlaylistInfo,
	YouTubeManagerStats,
	DownloaderStatus,
	MediaMode
} from 'addons/youtube/types';

// ===== Audio Quality Options =====

export const AUDIO_QUALITY_OPTIONS: { value: AudioQuality; label: string; description: string }[] =
	[
		{ value: 'best', label: 'Best', description: 'Highest available quality' },
		{ value: 'high', label: 'High', description: '~192 kbps' },
		{ value: 'medium', label: 'Medium', description: '~128 kbps' },
		{ value: 'low', label: 'Low', description: '~96 kbps' }
	];

// ===== Audio Format Options =====

export const AUDIO_FORMAT_OPTIONS: { value: AudioFormat; label: string; extension: string }[] = [
	{ value: 'aac', label: 'AAC (.m4a)', extension: 'm4a' },
	{ value: 'mp3', label: 'MP3', extension: 'mp3' },
	{ value: 'opus', label: 'Opus', extension: 'opus' }
];

// ===== Download Mode Options =====

export const DOWNLOAD_MODE_OPTIONS: { value: DownloadMode; label: string; description: string }[] =
	[
		{ value: 'both', label: 'Both', description: 'Download audio and video' },
		{ value: 'audio', label: 'Audio only', description: 'Download audio track only' },
		{ value: 'video', label: 'Video only', description: 'Download video with audio' }
	];

// ===== Video Quality Options =====

export const VIDEO_QUALITY_OPTIONS: { value: VideoQuality; label: string; description: string }[] =
	[
		{ value: 'best', label: 'Best', description: 'Highest available quality' },
		{ value: '1080p', label: '1080p', description: 'Full HD' },
		{ value: '720p', label: '720p', description: 'HD' },
		{ value: '480p', label: '480p', description: 'SD' }
	];

// ===== Video Format Options =====

export const VIDEO_FORMAT_OPTIONS: { value: VideoFormat; label: string; extension: string }[] = [
	{ value: 'mp4', label: 'MP4', extension: 'mp4' },
	{ value: 'mkv', label: 'MKV', extension: 'mkv' },
	{ value: 'webm', label: 'WebM', extension: 'webm' }
];

// ===== Subtitle Mode Options =====

export const SUBTITLE_MODE_OPTIONS: { value: SubtitleMode; label: string; description: string }[] =
	[
		{ value: 'none', label: 'None', description: "Don't download subtitles" },
		{ value: 'all', label: 'All', description: 'Download all available tracks' },
		{ value: 'selected', label: 'Selected', description: 'Choose specific languages' }
	];

// ===== Service State =====

export interface YouTubeServiceState {
	initialized: boolean;
	loading: boolean;
	error: string | null;
	libraryId?: string;
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

// ===== Settings (database) =====

export interface YouTubeSettings {
	id: ID;
	mediaMode?: MediaMode;
	downloadMode: DownloadMode;
	defaultQuality: AudioQuality;
	defaultFormat: AudioFormat;
	defaultVideoQuality: VideoQuality;
	defaultVideoFormat: VideoFormat;
	subtitleMode: SubtitleMode;
	subtitleLangs: string[];
	libraryId?: string;
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

// ===== UI Helper Functions =====

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
