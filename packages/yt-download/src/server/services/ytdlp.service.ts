import { spawn, execSync, type ChildProcess } from 'node:child_process';
import { createInterface } from 'node:readline';
import { existsSync, mkdirSync, writeFileSync, chmodSync } from 'node:fs';
import { join } from 'node:path';
import type { VideoInfo, PlaylistInfo, PlaylistVideo, AudioQuality, AudioFormat, DownloadMode, VideoQuality, VideoFormat } from '../../shared/types.js';
import { QUALITY_TO_YTDLP, FORMAT_TO_YTDLP, VIDEO_QUALITY_TO_YTDLP, VIDEO_FORMAT_TO_YTDLP } from '../../shared/constants.js';

const YTDLP_RELEASES_URL = 'https://github.com/yt-dlp/yt-dlp/releases/latest/download';

function getBinaryName(): string {
	switch (process.platform) {
		case 'win32':
			return 'yt-dlp.exe';
		case 'darwin':
			return 'yt-dlp_macos';
		case 'linux':
			return 'yt-dlp_linux';
		default:
			return '';
	}
}

export interface DownloadHandle {
	process: ChildProcess;
	onLine: (callback: (line: string) => void) => void;
	kill: () => void;
	waitForExit: () => Promise<number | null>;
}

export class YtDlpService {
	private binaryPath: string | null = null;
	private ffmpegPath: string | null = null;
	private baseDir: string = '';
	private available: boolean = false;

	initialize(baseDir: string): void {
		this.baseDir = baseDir;

		const binaryName = getBinaryName();
		if (!binaryName) return;

		const binaryPath = join(baseDir, binaryName);
		if (existsSync(binaryPath)) {
			this.binaryPath = binaryPath;
			this.available = true;
		}

		// Detect ffmpeg for video merging support
		try {
			this.ffmpegPath = execSync('which ffmpeg', { encoding: 'utf-8' }).trim();
		} catch {
			// ffmpeg not found
		}
	}

	isAvailable(): boolean {
		return this.available;
	}

	getBinaryPath(): string | null {
		return this.binaryPath;
	}

	async downloadBinary(): Promise<string> {
		const binaryName = getBinaryName();
		if (!binaryName) {
			throw new Error('yt-dlp is not supported on this platform');
		}

		if (!this.baseDir) {
			throw new Error('YtDlpService not initialized');
		}

		if (!existsSync(this.baseDir)) {
			mkdirSync(this.baseDir, { recursive: true });
		}

		const binaryPath = join(this.baseDir, binaryName);
		const downloadUrl = `${YTDLP_RELEASES_URL}/${binaryName}`;

		const response = await fetch(downloadUrl);
		if (!response.ok) {
			throw new Error(`Failed to download yt-dlp: HTTP ${response.status}`);
		}

		const buffer = Buffer.from(await response.arrayBuffer());
		writeFileSync(binaryPath, buffer);

		// Make executable on Unix
		if (process.platform !== 'win32') {
			chmodSync(binaryPath, 0o755);
		}

		this.binaryPath = binaryPath;
		this.available = true;

		return binaryPath;
	}

	async getVersion(): Promise<string> {
		if (!this.binaryPath) throw new Error('yt-dlp not available');

		return new Promise<string>((resolve, reject) => {
			const proc = spawn(this.binaryPath!, ['--version']);
			let stdout = '';
			proc.stdout.on('data', (data: Buffer) => {
				stdout += data.toString();
			});
			proc.on('close', (code) => {
				if (code !== 0) return reject(new Error('Failed to get yt-dlp version'));
				resolve(stdout.trim());
			});
			proc.on('error', reject);
		});
	}

	async getVideoInfo(url: string): Promise<VideoInfo> {
		if (!this.binaryPath) throw new Error('yt-dlp not available');

		return new Promise<VideoInfo>((resolve, reject) => {
			const proc = spawn(this.binaryPath!, ['-j', '--no-playlist', '--no-warnings', url]);
			let stdout = '';
			let stderr = '';
			proc.stdout.on('data', (data: Buffer) => {
				stdout += data.toString();
			});
			proc.stderr.on('data', (data: Buffer) => {
				stderr += data.toString();
			});
			proc.on('close', (code) => {
				if (code !== 0) return reject(new Error(`yt-dlp error: ${stderr}`));
				try {
					const info = JSON.parse(stdout);
					resolve({
						title: info.title ?? 'Unknown',
						duration: info.duration ?? 0,
						thumbnailUrl: info.thumbnail ?? null,
						uploader: info.uploader ?? null,
						videoId: info.id ?? ''
					});
				} catch (e) {
					reject(new Error(`Failed to parse yt-dlp JSON output: ${e}`));
				}
			});
			proc.on('error', reject);
		});
	}

	async getPlaylistInfo(url: string): Promise<PlaylistInfo> {
		if (!this.binaryPath) throw new Error('yt-dlp not available');

		return new Promise<PlaylistInfo>((resolve, reject) => {
			const proc = spawn(this.binaryPath!, ['-J', '--flat-playlist', '--no-warnings', url]);
			let stdout = '';
			let stderr = '';
			proc.stdout.on('data', (data: Buffer) => {
				stdout += data.toString();
			});
			proc.stderr.on('data', (data: Buffer) => {
				stderr += data.toString();
			});
			proc.on('close', (code) => {
				if (code !== 0) return reject(new Error(`yt-dlp error: ${stderr}`));
				try {
					const info = JSON.parse(stdout);
					if (info._type !== 'playlist') {
						return reject(new Error('URL is not a playlist'));
					}

					const entries: unknown[] = info.entries ?? [];
					const videos: PlaylistVideo[] = entries
						.map((entry: unknown, idx: number) => {
							const e = entry as Record<string, unknown>;
							const videoId = e.id as string | undefined;
							if (!videoId) return null;
							return {
								videoId,
								title: (e.title as string) ?? 'Unknown',
								duration: (e.duration as number) ?? 0,
								thumbnailUrl:
									Array.isArray(e.thumbnails) && e.thumbnails.length > 0
										? ((e.thumbnails[0] as Record<string, unknown>).url as string) ?? null
										: null,
								index: idx
							};
						})
						.filter((v): v is PlaylistVideo => v !== null);

					resolve({
						playlistId: info.id ?? '',
						title: info.title ?? 'Unknown Playlist',
						videoCount: videos.length,
						videos,
						thumbnailUrl:
							Array.isArray(info.thumbnails) && info.thumbnails.length > 0
								? info.thumbnails[0]?.url ?? null
								: null,
						author: info.uploader ?? null
					});
				} catch (e) {
					reject(new Error(`Failed to parse yt-dlp JSON output: ${e}`));
				}
			});
			proc.on('error', reject);
		});
	}

	spawnDownload(
		url: string,
		outputPath: string,
		quality: AudioQuality,
		format: AudioFormat,
		config?: { poToken?: string | null; cookies?: string | null },
		mode: DownloadMode = 'audio',
		videoQuality?: VideoQuality,
		videoFormat?: VideoFormat
	): DownloadHandle {
		if (!this.binaryPath) throw new Error('yt-dlp not available');

		const outputTemplate = join(outputPath, '%(title)s.%(ext)s');

		const args: string[] = [];

		if (mode === 'video') {
			const vq = videoQuality ?? 'best';
			const vf = videoFormat ?? 'mp4';
			args.push(
				'-f', VIDEO_QUALITY_TO_YTDLP[vq],
				'--merge-output-format', VIDEO_FORMAT_TO_YTDLP[vf]
			);
			if (this.ffmpegPath) {
				args.push('--ffmpeg-location', this.ffmpegPath);
			}
		} else {
			args.push(
				'-x',
				'--audio-format', FORMAT_TO_YTDLP[format],
				'--audio-quality', QUALITY_TO_YTDLP[quality]
			);
		}

		args.push(
			'-o', outputTemplate,
			'--newline',
			'--progress',
			'--no-playlist',
			'--no-warnings'
		);

		// Add PO token if configured
		if (config?.poToken) {
			args.push(
				'--extractor-args',
				`youtube:player_client=web;po_token=${config.poToken}`
			);
		}

		// Add cookies if configured
		if (config?.cookies) {
			// Write cookies to a temp file for yt-dlp
			const cookiesPath = join(outputPath, '.yt-cookies.txt');
			writeFileSync(cookiesPath, config.cookies);
			args.push('--cookies', cookiesPath);
		}

		args.push(url);

		const proc = spawn(this.binaryPath, args, {
			stdio: ['ignore', 'pipe', 'pipe']
		});

		const rl = createInterface({ input: proc.stdout! });

		return {
			process: proc,
			onLine: (callback: (line: string) => void) => {
				rl.on('line', callback);
			},
			kill: () => {
				proc.kill('SIGTERM');
			},
			waitForExit: () =>
				new Promise<number | null>((resolve) => {
					proc.on('close', (code) => resolve(code));
				})
		};
	}
}
