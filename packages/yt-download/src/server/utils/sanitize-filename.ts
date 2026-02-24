/** Sanitize a filename by replacing invalid characters with underscores. */
export function sanitizeFilename(name: string): string {
	return name
		.replace(/[/\\:*?"<>|]/g, '_')
		.trim();
}
