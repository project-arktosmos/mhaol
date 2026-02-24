/**
 * Parse a magnet URI to extract info_hash and display name.
 * Returns null if the URI is not a valid magnet link with a btih.
 */
export function parseMagnetUri(magnet: string): { infoHash: string; name: string } | null {
	if (!magnet.startsWith('magnet:')) return null;

	const url = new URL(magnet);
	const xt = url.searchParams.get('xt');
	if (!xt) return null;

	// Extract info hash from urn:btih:<hash>
	const btihMatch = xt.match(/^urn:btih:(.+)$/i);
	if (!btihMatch) return null;

	const infoHash = btihMatch[1].toLowerCase();

	// Extract display name, fallback to truncated hash
	const dn = url.searchParams.get('dn');
	const name = dn || `Torrent ${infoHash.slice(0, 8)}`;

	return { infoHash, name };
}
