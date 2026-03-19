export function formatSeeders(count: number): string {
	if (count >= 1000) return `${(count / 1000).toFixed(1)}k`;
	return String(count);
}

export function getSeedersColor(count: number): string {
	if (count >= 50) return 'text-success';
	if (count >= 10) return 'text-warning';
	if (count > 0) return 'text-error';
	return 'text-base-content/40';
}

export function formatSearchSize(bytes: number): string {
	if (bytes === 0) return '0 B';
	const units = ['B', 'KB', 'MB', 'GB', 'TB'];
	const i = Math.floor(Math.log(bytes) / Math.log(1024));
	const value = bytes / Math.pow(1024, i);
	return `${value.toFixed(i === 0 ? 0 : 1)} ${units[i]}`;
}

export function formatUploadDate(date: Date): string {
	const now = Date.now();
	const diff = now - date.getTime();
	const days = Math.floor(diff / (1000 * 60 * 60 * 24));

	if (days === 0) return 'Today';
	if (days === 1) return 'Yesterday';
	if (days < 30) return `${days}d ago`;
	if (days < 365) return `${Math.floor(days / 30)}mo ago`;
	return `${Math.floor(days / 365)}y ago`;
}
