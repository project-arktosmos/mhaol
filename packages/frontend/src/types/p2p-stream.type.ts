import type { ID } from '$types/core.type';

// ===== Video Codec =====

export type P2pVideoCodec = 'vp8' | 'vp9' | 'h264';

export const P2P_VIDEO_CODEC_OPTIONS: {
	value: P2pVideoCodec;
	label: string;
	description: string;
}[] = [
	{ value: 'vp8', label: 'VP8', description: 'Default, widely compatible' },
	{ value: 'vp9', label: 'VP9', description: 'Better compression, newer' },
	{ value: 'h264', label: 'H.264', description: 'Hardware-accelerated on most devices' }
];

// ===== Audio Codec =====

export type P2pAudioCodec = 'opus';

export const P2P_AUDIO_CODEC_OPTIONS: {
	value: P2pAudioCodec;
	label: string;
	description: string;
}[] = [{ value: 'opus', label: 'Opus', description: 'Only supported codec' }];

// ===== Video Quality =====

export type P2pVideoQuality = 'native' | '1080p' | '720p' | '480p' | '360p';

export const P2P_VIDEO_QUALITY_OPTIONS: {
	value: P2pVideoQuality;
	label: string;
	description: string;
}[] = [
	{ value: 'native', label: 'Native', description: 'Original resolution, no scaling' },
	{ value: '1080p', label: '1080p', description: 'Full HD (1920x1080)' },
	{ value: '720p', label: '720p', description: 'HD (1280x720)' },
	{ value: '480p', label: '480p', description: 'SD (854x480)' },
	{ value: '360p', label: '360p', description: 'Low (640x360)' }
];

// ===== Stream Mode =====

export type P2pStreamMode = 'audio' | 'video';

// ===== Settings (database-backed via settingsRepo) =====

export interface P2pStreamSettings {
	id: ID;
	stunServer: string;
	turnServers: string[];
	videoCodec: P2pVideoCodec;
	audioCodec: P2pAudioCodec;
	defaultStreamMode: P2pStreamMode;
	videoQuality: P2pVideoQuality;
}

// ===== Service State =====

export interface P2pStreamServiceState {
	initialized: boolean;
	loading: boolean;
	error: string | null;
	serverAvailable: boolean;
}
