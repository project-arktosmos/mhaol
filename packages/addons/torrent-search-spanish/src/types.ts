/**
 * Spanish-language torrent indexers tracked for integration.
 *
 * Source: https://github.com/Jackett/Jackett/issues/1468
 *
 * All entries below are PRIVATE invite-only trackers (or open-registration but
 * still requiring login) — search requires credentials. The Rust backend reads
 * credentials from per-indexer environment variables and silently skips any
 * indexer that has no credentials configured.
 */
export interface SpanishIndexerInfo {
	/** Stable identifier (lowercase, kebab-case). Used as env-var prefix. */
	id: string;
	/** Human-readable name. */
	name: string;
	/** Site URL. */
	url: string;
	/** Access policy. */
	access: 'private-invite' | 'private-open' | 'semi-private' | 'public';
	/** Content categories the tracker serves. */
	categories: Array<'movies' | 'tv' | 'anime' | 'music' | 'general' | 'hentai'>;
	/** Env var that holds the username. */
	usernameEnv: string;
	/** Env var that holds the password. */
	passwordEnv: string;
}

export const SPANISH_INDEXERS: SpanishIndexerInfo[] = [
	{
		id: 'hdspain',
		name: 'HDSpain',
		url: 'https://www.hd-spain.com/',
		access: 'private-invite',
		categories: ['movies', 'tv'],
		usernameEnv: 'HDSPAIN_USERNAME',
		passwordEnv: 'HDSPAIN_PASSWORD'
	},
	{
		id: 'hdcity',
		name: 'HDCity',
		url: 'https://hdcity.li/',
		access: 'private-invite',
		categories: ['movies', 'tv'],
		usernameEnv: 'HDCITY_USERNAME',
		passwordEnv: 'HDCITY_PASSWORD'
	},
	{
		id: 'hachede',
		name: 'HacheDe',
		url: 'https://hachede.me/',
		access: 'private-invite',
		categories: ['movies', 'tv'],
		usernameEnv: 'HACHEDE_USERNAME',
		passwordEnv: 'HACHEDE_PASSWORD'
	},
	{
		id: 'puntotorrent',
		name: 'Puntotorrent',
		url: 'https://xbt.puntotorrent.ch/',
		access: 'private-invite',
		categories: ['general', 'movies', 'tv', 'anime', 'music'],
		usernameEnv: 'PUNTOTORRENT_USERNAME',
		passwordEnv: 'PUNTOTORRENT_PASSWORD'
	},
	{
		id: 'torrentland',
		name: 'Torrentland',
		url: 'https://torrentland.li/',
		access: 'private-invite',
		categories: ['movies', 'tv'],
		usernameEnv: 'TORRENTLAND_USERNAME',
		passwordEnv: 'TORRENTLAND_PASSWORD'
	},
	{
		id: 'xbytesv2',
		name: 'xBytesV2',
		url: 'http://xbytesv2.li/',
		access: 'private-invite',
		categories: ['general', 'movies', 'tv', 'anime', 'music'],
		usernameEnv: 'XBYTESV2_USERNAME',
		passwordEnv: 'XBYTESV2_PASSWORD'
	},
	{
		id: 'unionfansub',
		name: 'Unionfansub',
		url: 'http://torrent.unionfansub.com/',
		access: 'private-open',
		categories: ['anime', 'hentai'],
		usernameEnv: 'UNIONFANSUB_USERNAME',
		passwordEnv: 'UNIONFANSUB_PASSWORD'
	}
];

/**
 * Spanish-language search keywords appended/substituted into queries
 * to surface Spanish-dub releases on language-agnostic indexers like PirateBay.
 */
export const SPANISH_QUERY_HINTS = ['castellano', 'español', 'latino'] as const;
