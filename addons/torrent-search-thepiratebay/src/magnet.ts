const MAGNET_TRACKERS = [
	'udp://tracker.opentrackr.org:1337/announce',
	'udp://tracker.openbittorrent.com:6969/announce',
	'udp://open.stealth.si:80/announce',
	'udp://tracker.torrent.eu.org:451/announce',
	'udp://tracker.dler.org:6969/announce',
	'udp://opentracker.i2p.rocks:6969/announce'
];

export function buildMagnetLink(infoHash: string, name: string): string {
	const encodedName = encodeURIComponent(name);
	const trackerParams = MAGNET_TRACKERS.map((t) => `&tr=${encodeURIComponent(t)}`).join('');
	return `magnet:?xt=urn:btih:${infoHash}&dn=${encodedName}${trackerParams}`;
}
