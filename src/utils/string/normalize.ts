export default function normalize(input: string): string {
	return input
		.trim()
		.replace(/<[^>]*>/g, '')
		.toLowerCase()
		.split(' ')
		.join('-');
}