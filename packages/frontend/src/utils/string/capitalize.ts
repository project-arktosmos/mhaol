export default function capitalize(input?: string | null): string {
	if (!input) return '';
	return (
		input
			.replace(/([a-z])([A-Z])/g, '$1 $2')
			.replace(/-/g, ' ')
			.split(' ')
			.map((word) => word.charAt(0).toUpperCase() + word.slice(1).toLowerCase())
			.join(' ')
	);
}