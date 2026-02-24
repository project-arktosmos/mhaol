import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { readdir } from 'node:fs/promises';
import { resolve, dirname, join } from 'node:path';
import { homedir } from 'node:os';
import type { BrowseDirectoryResponse, DirectoryEntry } from '$types/library.type';

export const GET: RequestHandler = async ({ url }) => {
	const requestedPath = url.searchParams.get('path');
	const targetPath = requestedPath ? resolve(requestedPath) : homedir();

	try {
		const entries = await readdir(targetPath, { withFileTypes: true });

		const directories: DirectoryEntry[] = entries
			.filter((entry) => entry.isDirectory() && !entry.name.startsWith('.'))
			.sort((a, b) => a.name.localeCompare(b.name))
			.map((entry) => ({
				name: entry.name,
				path: join(targetPath, entry.name)
			}));

		const parent = targetPath !== '/' ? dirname(targetPath) : null;

		const response: BrowseDirectoryResponse = {
			path: targetPath,
			parent,
			directories
		};

		return json(response);
	} catch (err) {
		const code = (err as NodeJS.ErrnoException).code;
		if (code === 'ENOENT') {
			return json({ error: 'Directory not found' }, { status: 404 });
		}
		if (code === 'EACCES' || code === 'EPERM') {
			return json({ error: 'Permission denied' }, { status: 403 });
		}
		return json({ error: err instanceof Error ? err.message : String(err) }, { status: 500 });
	}
};
