export function extractYear(dateString: string | undefined): string {
	if (!dateString) return 'Unknown';
	return dateString.split('-')[0] || 'Unknown';
}
