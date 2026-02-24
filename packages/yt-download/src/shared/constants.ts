import type { AudioQuality, AudioFormat, VideoQuality, VideoFormat } from './types.js';

export const AUDIO_QUALITY_OPTIONS: { value: AudioQuality; label: string; description: string }[] =
	[
		{ value: 'best', label: 'Best', description: 'Highest available quality' },
		{ value: 'high', label: 'High', description: '~192 kbps' },
		{ value: 'medium', label: 'Medium', description: '~128 kbps' },
		{ value: 'low', label: 'Low', description: '~96 kbps' }
	];

export const AUDIO_FORMAT_OPTIONS: { value: AudioFormat; label: string; extension: string }[] = [
	{ value: 'aac', label: 'AAC (.m4a)', extension: 'm4a' },
	{ value: 'mp3', label: 'MP3', extension: 'mp3' },
	{ value: 'opus', label: 'Opus', extension: 'opus' }
];

/** Map audio quality to yt-dlp --audio-quality argument */
export const QUALITY_TO_YTDLP: Record<AudioQuality, string> = {
	best: '0',
	high: '2',
	medium: '5',
	low: '9'
};

/** Map audio format to yt-dlp --audio-format argument */
export const FORMAT_TO_YTDLP: Record<AudioFormat, string> = {
	aac: 'm4a',
	mp3: 'mp3',
	opus: 'opus'
};

/** Map audio format to file extension */
export const FORMAT_TO_EXTENSION: Record<AudioFormat, string> = {
	aac: 'm4a',
	mp3: 'mp3',
	opus: 'opus'
};

// ===== Video Constants =====

export const VIDEO_QUALITY_OPTIONS: { value: VideoQuality; label: string; description: string }[] =
	[
		{ value: 'best', label: 'Best', description: 'Highest available quality' },
		{ value: '1080p', label: '1080p', description: 'Full HD' },
		{ value: '720p', label: '720p', description: 'HD' },
		{ value: '480p', label: '480p', description: 'SD' }
	];

export const VIDEO_FORMAT_OPTIONS: { value: VideoFormat; label: string; extension: string }[] = [
	{ value: 'mp4', label: 'MP4', extension: 'mp4' },
	{ value: 'mkv', label: 'MKV', extension: 'mkv' },
	{ value: 'webm', label: 'WebM', extension: 'webm' }
];

/**
 * Map video quality to yt-dlp -f format selector.
 * Uses bv* (includes muxed formats) so a single pre-muxed stream can match
 * as a fallback when ffmpeg is not available for merging separate streams.
 * Fallback chain: mp4 streams -> any streams (needs ffmpeg merge) -> best pre-muxed single stream.
 */
export const VIDEO_QUALITY_TO_YTDLP: Record<VideoQuality, string> = {
	best: 'bv*[ext=mp4]+ba[ext=m4a]/bv*+ba/b',
	'1080p': 'bv*[height<=1080][ext=mp4]+ba[ext=m4a]/bv*[height<=1080]+ba/b[height<=1080]/b',
	'720p': 'bv*[height<=720][ext=mp4]+ba[ext=m4a]/bv*[height<=720]+ba/b[height<=720]/b',
	'480p': 'bv*[height<=480][ext=mp4]+ba[ext=m4a]/bv*[height<=480]+ba/b[height<=480]/b'
};

/** Map video format to yt-dlp --merge-output-format argument */
export const VIDEO_FORMAT_TO_YTDLP: Record<VideoFormat, string> = {
	mp4: 'mp4',
	mkv: 'mkv',
	webm: 'webm'
};
