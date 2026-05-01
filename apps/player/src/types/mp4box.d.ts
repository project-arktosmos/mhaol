/**
 * Minimal type surface for `mp4box` that we actually use. The full library
 * has no published types; this only covers the streaming/segmentation API
 * we drive in `src/ipfs/stream-player.ts`.
 */
declare module 'mp4box' {
	export interface Mp4BoxTrack {
		id: number;
		codec: string;
		type?: string;
		movie_duration?: number;
		duration?: number;
		timescale?: number;
		language?: string;
		nb_samples?: number;
		video?: { width: number; height: number };
		audio?: { sample_rate: number; channel_count: number };
	}

	export interface Mp4BoxInfo {
		duration: number;
		timescale: number;
		isFragmented: boolean;
		isProgressive: boolean;
		hasIOD: boolean;
		brands: string[];
		created: Date;
		modified: Date;
		tracks: Mp4BoxTrack[];
		mime?: string;
	}

	export interface Mp4BoxFile {
		onReady?: (info: Mp4BoxInfo) => void;
		onError?: (msg: string) => void;
		onSegment?: (
			id: number,
			user: unknown,
			buffer: ArrayBuffer,
			sampleNum: number,
			last: boolean
		) => void;
		appendBuffer(data: ArrayBuffer & { fileStart: number }): number;
		setSegmentOptions(
			id: number,
			user: unknown,
			options: { nbSamples?: number; rapAlignement?: boolean }
		): void;
		initializeSegmentation(): { id: number; user: unknown; buffer: ArrayBuffer }[];
		start(): void;
		stop(): void;
		flush(): void;
	}

	export function createFile(keepMdatData?: boolean): Mp4BoxFile;

	const _default: { createFile: typeof createFile };
	export default _default;
}
