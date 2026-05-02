import type { SubsLyricsItem } from '$types/subs-lyrics.type';

// Tokens that meaningfully identify which release a subtitle is timed
// to. Resolution + source + codec + audio. We deliberately leave out
// the year and the movie title itself: the torrent and the sub are
// already known to be for the same movie (they were both fetched off
// the same TMDB id), so matching on those would inflate every score.
const TELLTALE_TOKENS = new Set([
	'480p',
	'576p',
	'720p',
	'1080p',
	'1440p',
	'2160p',
	'4k',
	'uhd',
	'bluray',
	'blu-ray',
	'bdrip',
	'brrip',
	'br-rip',
	'web-dl',
	'webdl',
	'webrip',
	'web',
	'hdrip',
	'dvdrip',
	'dvd',
	'hdtv',
	'hdcam',
	'cam',
	'ts',
	'tc',
	'tvrip',
	'r5',
	'r6',
	'x264',
	'x265',
	'h264',
	'h.264',
	'h265',
	'h.265',
	'hevc',
	'av1',
	'xvid',
	'divx',
	'aac',
	'ac3',
	'eac3',
	'ddp5.1',
	'dd5.1',
	'5.1',
	'7.1',
	'dts',
	'dts-hd',
	'flac',
	'atmos',
	'truehd',
	'opus',
	'mp3',
	'hdr',
	'hdr10',
	'hdr10+',
	'dv',
	'dovi',
	'imax',
	'remux',
	'proper',
	'repack',
	'extended',
	'directors',
	'unrated'
]);

const FILE_EXT_RE = /\.(srt|ass|ssa|sub|idx|vtt|smi|mkv|mp4|avi|m4v|mov|webm|ts|m2ts|wmv|flv)$/i;

function tokenize(value: string): string[] {
	return value
		.toLowerCase()
		.split(/[.\s\-_(),[\]+]+/)
		.filter((t) => t.length > 0);
}

function extractGroup(value: string): string | null {
	const cleaned = value.replace(FILE_EXT_RE, '');
	const match = cleaned.match(/-([A-Za-z0-9]{2,})$/);
	return match ? match[1].toLowerCase() : null;
}

export interface MatchedSub {
	sub: SubsLyricsItem;
	score: number;
	overlap: string[];
	groupMatched: boolean;
}

export function matchSubsToTorrent(
	torrentTitle: string | undefined | null,
	subs: SubsLyricsItem[]
): MatchedSub[] {
	if (!torrentTitle) return [];
	const torrentTokens = tokenize(torrentTitle);
	const torrentTellTales = new Set(torrentTokens.filter((t) => TELLTALE_TOKENS.has(t)));
	const torrentGroup = extractGroup(torrentTitle);
	if (torrentTellTales.size === 0 && !torrentGroup) return [];

	const out: MatchedSub[] = [];
	for (const sub of subs) {
		const release = sub.release;
		if (!release) continue;
		const subTokens = tokenize(release);
		const subTellTales = new Set(subTokens.filter((t) => TELLTALE_TOKENS.has(t)));
		const subGroup = extractGroup(release);

		const overlap: string[] = [];
		for (const t of torrentTellTales) {
			if (subTellTales.has(t)) overlap.push(t);
		}
		const groupMatched = !!torrentGroup && torrentGroup === subGroup;

		// Group match is the strongest signal — subbers explicitly time
		// against a specific release group, so a shared trailing `-TAG`
		// almost always means correct sync. Falling back: require at
		// least 2 telltale tokens (e.g. "1080p" + "bluray") to claim a
		// match. A single shared "1080p" or "x264" alone is too noisy.
		let score = 0;
		if (groupMatched) score += 100;
		score += overlap.length;
		const qualifies = groupMatched || overlap.length >= 2;
		if (!qualifies) continue;
		out.push({ sub, score, overlap, groupMatched });
	}

	const LANGUAGE_PRIORITY: Record<string, number> = {
		English: 0,
		Catalan: 1,
		Spanish: 2
	};
	out.sort((a, b) => {
		if (b.score !== a.score) return b.score - a.score;
		const al = a.sub.display ?? a.sub.language ?? '';
		const bl = b.sub.display ?? b.sub.language ?? '';
		const ar = LANGUAGE_PRIORITY[al] ?? 99;
		const br = LANGUAGE_PRIORITY[bl] ?? 99;
		if (ar !== br) return ar - br;
		return al.localeCompare(bl);
	});
	return out;
}
