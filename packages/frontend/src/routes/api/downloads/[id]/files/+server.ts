import { json, error } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { readdirSync, statSync } from 'node:fs';
import { join } from 'node:path';

interface FileEntry {
	name: string;
	size: number;
	isDirectory: boolean;
}

function listFiles(dirPath: string): FileEntry[] {
	try {
		const entries = readdirSync(dirPath);
		return entries.map((name) => {
			const fullPath = join(dirPath, name);
			const stat = statSync(fullPath);
			return {
				name,
				size: stat.size,
				isDirectory: stat.isDirectory()
			};
		});
	} catch {
		return [];
	}
}

export const GET: RequestHandler = async ({ params, url, locals }) => {
	const id = params.id;
	const type = url.searchParams.get('type');

	if (type === 'youtube') {
		const row = locals.youtubeDownloadRepo.get(id);
		if (!row) throw error(404, 'Download not found');

		return json({
			type: 'youtube',
			thumbnailUrl: row.thumbnail_url,
			title: row.title,
			url: row.url,
			videoId: row.video_id,
			mode: row.mode,
			quality: row.quality,
			format: row.format,
			durationSeconds: row.duration_seconds,
			outputPath: row.output_path
		});
	}

	if (type === 'torrent') {
		const row = locals.torrentDownloadRepo.get(id);
		if (!row) throw error(404, 'Download not found');

		const torrentDir = row.output_path && row.name ? `${row.output_path}/${row.name}` : null;
		const files = torrentDir ? listFiles(torrentDir) : [];

		return json({
			type: 'torrent',
			name: row.name,
			directory: torrentDir,
			files
		});
	}

	throw error(400, 'Missing or invalid type parameter');
};
