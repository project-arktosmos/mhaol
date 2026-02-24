import { json } from '@sveltejs/kit';
import { readdirSync, statSync } from 'node:fs';
import { join, extname, basename } from 'node:path';
import type { RequestHandler } from './$types';
import type { PlayableFile } from '$types/player.type';

const VIDEO_EXTENSIONS = new Set(['.mp4', '.mkv', '.avi', '.webm', '.mov', '.m4v', '.wmv', '.flv']);
const AUDIO_EXTENSIONS = new Set(['.mp3', '.flac', '.aac', '.opus', '.ogg', '.wav', '.m4a', '.wma']);
const MEDIA_EXTENSIONS = new Set([...VIDEO_EXTENSIONS, ...AUDIO_EXTENSIONS]);

function findMediaFiles(dir: string): string[] {
	const results: string[] = [];
	try {
		const entries = readdirSync(dir, { withFileTypes: true });
		for (const entry of entries) {
			const fullPath = join(dir, entry.name);
			if (entry.isDirectory()) {
				results.push(...findMediaFiles(fullPath));
			} else if (entry.isFile() && MEDIA_EXTENSIONS.has(extname(entry.name).toLowerCase())) {
				results.push(fullPath);
			}
		}
	} catch {
		// Directory may not exist or be inaccessible
	}
	return results;
}

function modeFromExtension(filePath: string): 'audio' | 'video' {
	return AUDIO_EXTENSIONS.has(extname(filePath).toLowerCase()) ? 'audio' : 'video';
}

export const GET: RequestHandler = async ({ locals }) => {
	const youtubeRows = locals.youtubeDownloadRepo.getByState('completed');
	const torrentRows = locals.torrentDownloadRepo
		.getAll()
		.filter((r) => r.state === 'seeding' || r.progress >= 1);

	const playable: PlayableFile[] = [];

	for (const row of youtubeRows) {
		if (!row.output_path) continue;
		playable.push({
			id: row.download_id,
			type: 'youtube',
			name: row.title,
			outputPath: row.output_path,
			mode: row.mode as 'audio' | 'video',
			format: row.format,
			videoFormat: row.video_format,
			thumbnailUrl: row.thumbnail_url,
			durationSeconds: row.duration_seconds,
			size: row.total_bytes,
			completedAt: row.updated_at
		});
	}

	for (const row of torrentRows) {
		if (!row.output_path) continue;

		const torrentPath =
			row.output_path && row.name ? join(row.output_path, row.name) : row.output_path;

		let mediaFiles: string[] = [];
		try {
			const stat = statSync(torrentPath);
			if (stat.isDirectory()) {
				mediaFiles = findMediaFiles(torrentPath);
			} else if (stat.isFile() && MEDIA_EXTENSIONS.has(extname(torrentPath).toLowerCase())) {
				mediaFiles = [torrentPath];
			}
		} catch {
			continue;
		}

		for (const filePath of mediaFiles) {
			let fileSize = 0;
			try {
				fileSize = statSync(filePath).size;
			} catch {
				// Skip if stat fails
			}

			playable.push({
				id: `${row.info_hash}:${basename(filePath)}`,
				type: 'torrent',
				name: basename(filePath),
				outputPath: filePath,
				mode: modeFromExtension(filePath),
				format: extname(filePath).slice(1).toLowerCase(),
				videoFormat: null,
				thumbnailUrl: null,
				durationSeconds: null,
				size: fileSize,
				completedAt: row.updated_at
			});
		}
	}

	playable.sort((a, b) => new Date(b.completedAt).getTime() - new Date(a.completedAt).getTime());

	return json(playable);
};
