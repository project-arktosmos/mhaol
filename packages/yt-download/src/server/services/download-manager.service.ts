import { existsSync, mkdirSync, readdirSync, statSync } from 'node:fs';
import { join, extname, basename, parse as parsePath } from 'node:path';
import type { ChildProcess } from 'node:child_process';
import type {
	AudioQuality,
	AudioFormat,
	DownloadProgress,
	DownloadState,
	DownloadMode,
	VideoQuality,
	VideoFormat,
	ManagerStats,
	YtDownloadConfig
} from '../../shared/types.js';
import { YtDlpService } from './ytdlp.service.js';
import { SSEBroadcasterService } from './sse-broadcaster.service.js';
import {
	parseProgressPercent,
	parseDestination,
	parseExtractAudioDestination,
	parseMergerDestination,
	parseAlreadyDownloaded
} from '../utils/parse-progress.js';

interface QueuedDownload {
	downloadId: string;
	url: string;
	videoId: string;
	title: string;
	mode: DownloadMode;
	quality: AudioQuality;
	format: AudioFormat;
	videoQuality?: VideoQuality;
	videoFormat?: VideoFormat;
}

type PersistenceCallback = (progress: DownloadProgress) => void;
type DeleteCallback = (downloadIds: string[]) => void;

export class DownloadManagerService {
	private downloads: Map<string, DownloadProgress> = new Map();
	private queue: QueuedDownload[] = [];
	private activeProcesses: Map<string, ChildProcess> = new Map();
	private queueProcessorRunning = false;
	private downloadCounter = 0;
	private persistenceCallback: PersistenceCallback | null = null;
	private deleteCallback: DeleteCallback | null = null;

	private config: YtDownloadConfig = {
		outputPath: join(process.cwd(), 'downloads'),
		defaultQuality: 'high',
		defaultFormat: 'aac',
		poToken: null,
		cookies: null
	};

	constructor(
		private ytdlp: YtDlpService,
		private broadcaster: SSEBroadcasterService
	) {}

	setPersistenceCallback(cb: PersistenceCallback): void {
		this.persistenceCallback = cb;
	}

	setDeleteCallback(cb: DeleteCallback): void {
		this.deleteCallback = cb;
	}

	initialize(outputPath?: string): void {
		if (outputPath) {
			this.config.outputPath = outputPath;
		}
		if (!existsSync(this.config.outputPath)) {
			mkdirSync(this.config.outputPath, { recursive: true });
		}
	}

	// ===== Config =====

	getConfig(): YtDownloadConfig {
		return { ...this.config };
	}

	updateConfig(updates: Partial<YtDownloadConfig>): void {
		Object.assign(this.config, updates);
		if (updates.outputPath && !existsSync(updates.outputPath)) {
			mkdirSync(updates.outputPath, { recursive: true });
		}
	}

	// ===== Download Queue =====

	queueDownload(request: {
		url: string;
		videoId: string;
		title: string;
		mode?: DownloadMode;
		quality?: AudioQuality;
		format?: AudioFormat;
		videoQuality?: VideoQuality;
		videoFormat?: VideoFormat;
	}): string {
		const downloadId = this.generateDownloadId();
		const mode = request.mode ?? 'audio';
		const quality = request.quality ?? this.config.defaultQuality;
		const format = request.format ?? this.config.defaultFormat;

		const progress: DownloadProgress = {
			downloadId,
			url: request.url,
			videoId: request.videoId,
			title: request.title,
			state: 'pending',
			progress: 0,
			downloadedBytes: 0,
			totalBytes: 0,
			outputPath: null,
			error: null,
			mode,
			quality,
			format,
			videoQuality: request.videoQuality ?? null,
			videoFormat: request.videoFormat ?? null,
			thumbnailUrl: null,
			durationSeconds: null
		};

		this.downloads.set(downloadId, progress);
		this.persistenceCallback?.(progress);

		this.queue.push({
			downloadId,
			url: request.url,
			videoId: request.videoId,
			title: request.title,
			mode,
			quality,
			format,
			videoQuality: request.videoQuality,
			videoFormat: request.videoFormat
		});

		this.broadcaster.broadcastProgress(progress);
		this.startQueueProcessor();

		return downloadId;
	}

	queuePlaylistDownloads(request: {
		videos: { url: string; videoId: string; title: string }[];
		mode?: DownloadMode;
		quality?: AudioQuality;
		format?: AudioFormat;
		videoQuality?: VideoQuality;
		videoFormat?: VideoFormat;
	}): string[] {
		const ids: string[] = [];
		for (const video of request.videos) {
			const id = this.queueDownload({
				...video,
				mode: request.mode,
				quality: request.quality,
				format: request.format,
				videoQuality: request.videoQuality,
				videoFormat: request.videoFormat
			});
			ids.push(id);
		}
		return ids;
	}

	// ===== Queue Processor =====

	private startQueueProcessor(): void {
		if (this.queueProcessorRunning) return;
		this.queueProcessorRunning = true;
		this.processQueue();
	}

	private async processQueue(): Promise<void> {
		while (this.queue.length > 0) {
			const queued = this.queue.shift()!;
			await this.runDownload(queued);
			// Small delay between downloads to be nice to YouTube
			await new Promise((resolve) => setTimeout(resolve, 500));
		}
		this.queueProcessorRunning = false;
	}

	private async runDownload(queued: QueuedDownload): Promise<void> {
		const { downloadId } = queued;

		// Update state to Fetching
		this.updateDownloadState(downloadId, { state: 'fetching' });

		try {
			const handle = this.ytdlp.spawnDownload(
				queued.url,
				this.config.outputPath,
				queued.quality,
				queued.format,
				{ poToken: this.config.poToken, cookies: this.config.cookies },
				queued.mode,
				queued.videoQuality,
				queued.videoFormat
			);

			this.activeProcesses.set(downloadId, handle.process);

			let finalFilename: string | null = null;

			handle.onLine((line: string) => {
				// Parse progress percentage
				if (line.includes('%')) {
					const percent = parseProgressPercent(line);
					if (percent !== null) {
						this.updateDownloadState(downloadId, {
							state: 'downloading',
							progress: percent / 100
						});
					}
				}

				// Capture download destination
				const dest = parseDestination(line);
				if (dest) {
					const parsed = parsePath(dest);
					this.updateDownloadState(downloadId, { title: parsed.name });
					finalFilename = dest;
				}

				// Capture extracted audio destination (final file)
				const extractDest = parseExtractAudioDestination(line);
				if (extractDest) {
					finalFilename = extractDest;
				}

				// Capture merged video destination (final file)
				const mergerDest = parseMergerDestination(line);
				if (mergerDest) {
					finalFilename = mergerDest;
				}

				// Already downloaded
				const alreadyPath = parseAlreadyDownloaded(line);
				if (alreadyPath) {
					finalFilename = alreadyPath;
				}
			});

			const exitCode = await handle.waitForExit();
			this.activeProcesses.delete(downloadId);

			if (exitCode !== 0) {
				// Check if it was cancelled
				const current = this.downloads.get(downloadId);
				if (current?.state === 'cancelled') return;

				this.updateDownloadState(downloadId, {
					state: 'failed',
					error: `yt-dlp exited with code ${exitCode}`
				});
				return;
			}

			// Find the output file
			let outputPath: string | null = finalFilename;
			if (!outputPath || !existsSync(outputPath)) {
				outputPath = this.findNewestFile(this.config.outputPath, queued.mode);
			}

			if (outputPath) {
				this.updateDownloadState(downloadId, {
					state: 'completed',
					progress: 1,
					outputPath
				});
			} else {
				this.updateDownloadState(downloadId, {
					state: 'failed',
					error: 'Could not find downloaded file'
				});
			}
		} catch (err) {
			this.activeProcesses.delete(downloadId);
			const current = this.downloads.get(downloadId);
			if (current?.state === 'cancelled') return;

			this.updateDownloadState(downloadId, {
				state: 'failed',
				error: err instanceof Error ? err.message : String(err)
			});
		}
	}

	// ===== Download Management =====

	cancelDownload(downloadId: string): void {
		const proc = this.activeProcesses.get(downloadId);
		if (proc) {
			proc.kill('SIGTERM');
			this.activeProcesses.delete(downloadId);
		}

		// Also remove from queue if pending
		this.queue = this.queue.filter((q) => q.downloadId !== downloadId);

		this.updateDownloadState(downloadId, { state: 'cancelled' });
	}

	clearCompleted(): void {
		const deletedIds: string[] = [];
		for (const [id, progress] of this.downloads) {
			if (
				progress.state === 'completed' ||
				progress.state === 'failed' ||
				progress.state === 'cancelled'
			) {
				this.downloads.delete(id);
				deletedIds.push(id);
			}
		}
		if (deletedIds.length > 0) {
			this.deleteCallback?.(deletedIds);
		}
		this.broadcaster.broadcastStats(this.getStats());
	}

	clearQueue(): void {
		const clearedIds = this.queue.map((q) => q.downloadId);
		this.queue = [];

		for (const id of clearedIds) {
			this.updateDownloadState(id, { state: 'cancelled' });
		}
	}

	getProgress(downloadId: string): DownloadProgress | null {
		return this.downloads.get(downloadId) ?? null;
	}

	getAllProgress(): DownloadProgress[] {
		return Array.from(this.downloads.values());
	}

	getStats(): ManagerStats {
		let active = 0;
		let queued = 0;
		let completed = 0;
		let failed = 0;

		for (const p of this.downloads.values()) {
			switch (p.state) {
				case 'pending':
					queued++;
					break;
				case 'fetching':
				case 'downloading':
					active++;
					break;
				case 'completed':
					completed++;
					break;
				case 'failed':
				case 'cancelled':
					failed++;
					break;
			}
		}

		let ytdlpVersion: string | null = null;
		// Version is fetched asynchronously, we cache it in stats calls
		// It's populated by the status route

		return {
			activeDownloads: active,
			queuedDownloads: queued,
			completedDownloads: completed,
			failedDownloads: failed,
			ytdlpAvailable: this.ytdlp.isAvailable(),
			ytdlpVersion
		};
	}

	// ===== Internal Helpers =====

	private generateDownloadId(): string {
		this.downloadCounter++;
		return `yt-${Date.now()}-${this.downloadCounter}`;
	}

	private updateDownloadState(
		downloadId: string,
		updates: Partial<DownloadProgress>
	): void {
		const current = this.downloads.get(downloadId);
		if (!current) return;

		Object.assign(current, updates);
		this.broadcaster.broadcastProgress(current);
		this.broadcaster.broadcastStats(this.getStats());
		this.persistenceCallback?.(current);
	}

	private findNewestFile(dir: string, mode: DownloadMode = 'audio'): string | null {
		const audioExtensions = new Set(['.m4a', '.mp3', '.opus', '.webm']);
		const videoExtensions = new Set(['.mp4', '.mkv', '.webm']);
		const extensions = mode === 'video' ? videoExtensions : audioExtensions;
		let newestFile: string | null = null;
		let newestTime = 0;

		try {
			const entries = readdirSync(dir);
			for (const entry of entries) {
				const fullPath = join(dir, entry);
				const ext = extname(entry).toLowerCase();
				if (!extensions.has(ext)) continue;

				try {
					const stat = statSync(fullPath);
					if (stat.mtimeMs > newestTime) {
						newestTime = stat.mtimeMs;
						newestFile = fullPath;
					}
				} catch {
					// skip files we can't stat
				}
			}
		} catch {
			// directory not readable
		}

		return newestFile;
	}
}
