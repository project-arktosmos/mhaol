/** Parse progress percentage from a yt-dlp stdout line. Returns 0-100 or null. */
export function parseProgressPercent(line: string): number | null {
	for (const part of line.split(/\s+/)) {
		if (part.endsWith('%')) {
			const num = parseFloat(part.slice(0, -1));
			if (!isNaN(num)) return num;
		}
	}
	return null;
}

/** Parse the download destination path from a yt-dlp stdout line. */
export function parseDestination(line: string): string | null {
	// [download] Destination: /path/to/file.ext
	if (line.includes('[download] Destination:')) {
		const dest = line.split('Destination:')[1];
		return dest ? dest.trim() : null;
	}
	return null;
}

/** Parse the extracted audio destination from a yt-dlp stdout line. */
export function parseExtractAudioDestination(line: string): string | null {
	// [ExtractAudio] Destination: /path/to/file.ext
	if (line.includes('[ExtractAudio] Destination:')) {
		const dest = line.split('Destination:')[1];
		return dest ? dest.trim() : null;
	}
	return null;
}

/** Parse the merged video destination from a yt-dlp stdout line. */
export function parseMergerDestination(line: string): string | null {
	// [Merger] Merging formats into "/path/to/file.mp4"
	if (line.includes('[Merger] Merging formats into')) {
		const match = line.match(/into "(.+)"/);
		return match ? match[1] : null;
	}
	return null;
}

/** Parse an "already downloaded" line to extract the file path. */
export function parseAlreadyDownloaded(line: string): string | null {
	if (!line.includes('has already been downloaded')) return null;
	// Format: [download] [/path/to/file.ext] has already been downloaded
	// Find the second '[' and its matching ']'
	const firstClose = line.indexOf(']');
	if (firstClose === -1) return null;
	const secondOpen = line.indexOf('[', firstClose + 1);
	if (secondOpen === -1) return null;
	const secondClose = line.indexOf(']', secondOpen + 1);
	if (secondClose === -1) return null;
	return line.slice(secondOpen + 1, secondClose);
}
