import { getTransport } from './transport-context';

export function resolveImageUrl(path: string): string {
	return getTransport().resolveUrl(path);
}

export async function resolveImageUrlAsync(path: string): Promise<string> {
	return getTransport().resolveUrlAsync(path);
}
