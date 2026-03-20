import type { TorrentAnalysis } from 'frontend/types/smart-search.type';

const QUALITY_PATTERNS: [RegExp, string][] = [
	[/2160p/i, '2160p'],
	[/4K/i, '4K'],
	[/UHD/i, '4K UHD'],
	[/1080p/i, '1080p'],
	[/720p/i, '720p'],
	[/480p/i, '480p'],
	[/360p/i, '360p']
];

const SOURCE_PATTERNS: [RegExp, string][] = [
	[/blu[\s.-]?ray/i, 'BluRay'],
	[/bdrip/i, 'BDRip'],
	[/brrip/i, 'BRRip'],
	[/web[\s.-]?dl/i, 'WEB-DL'],
	[/web[\s.-]?rip/i, 'WEBRip'],
	[/webrip/i, 'WEBRip'],
	[/hdtv/i, 'HDTV'],
	[/dvd[\s.-]?rip/i, 'DVDRip'],
	[/dvdscr/i, 'DVDScr'],
	[/hdcam/i, 'HDCam'],
	[/cam[\s.-]?rip/i, 'CAMRip'],
	[/telesync|ts(?=[\s.])/i, 'TS'],
	[/hdrip/i, 'HDRip']
];

const AUDIO_QUALITY_PATTERNS: [RegExp, string][] = [
	[/\bFLAC\b/i, 'FLAC'],
	[/\bALAC\b/i, 'ALAC'],
	[/\blossless\b/i, 'Lossless'],
	[/\bDSD\b/i, 'DSD'],
	[/\bWAV\b/i, 'WAV'],
	[/\b24[\s-]?bit\b/i, '24-bit'],
	[/\b16[\s-]?bit\b/i, '16-bit'],
	[/\b320\s?k(?:bps?)?\b/i, '320kbps'],
	[/\b256\s?k(?:bps?)?\b/i, '256kbps'],
	[/\b192\s?k(?:bps?)?\b/i, '192kbps'],
	[/\b128\s?k(?:bps?)?\b/i, '128kbps'],
	[/\bMP3\b/i, 'MP3'],
	[/\bAAC\b/i, 'AAC'],
	[/\bOGG\b/i, 'OGG'],
	[/\bVORBIS\b/i, 'Vorbis'],
	[/\bOPUS\b/i, 'Opus']
];

const LANGUAGE_PATTERNS: [RegExp, string][] = [
	[/\bmulti(?:[\s.-]?(?:lang|audio|sub))?s?\b/i, 'Multi'],
	[/\bdual[\s.-]?audio\b/i, 'Dual Audio'],
	[/\bengl?(?:ish)?\b/i, 'English'],
	[/\bspan(?:ish)?\b|\bESP\b/i, 'Spanish'],
	[/\bfrench\b|\bFR\b|\bVFF\b|\bTRUEFRENCH\b/i, 'French'],
	[/\bgerman\b|\bDE\b/i, 'German'],
	[/\bitalian\b|\bITA\b/i, 'Italian'],
	[/\bportugu[eê]se?\b|\bPT[\s.-]?BR\b/i, 'Portuguese'],
	[/\brussian\b|\bRUS\b/i, 'Russian'],
	[/\bjapanese\b|\bJPN?\b/i, 'Japanese'],
	[/\bkorean\b|\bKOR\b/i, 'Korean'],
	[/\bchinese\b|\bCHI\b|\bCHS\b|\bCHT\b/i, 'Chinese'],
	[/\bhindi\b|\bHIN\b/i, 'Hindi'],
	[/\barabic\b|\bARA\b/i, 'Arabic'],
	[/\bdutch\b|\bNLD?\b/i, 'Dutch'],
	[/\bswedish\b|\bSWE\b/i, 'Swedish'],
	[/\bnorwegian\b|\bNOR\b/i, 'Norwegian'],
	[/\bdanish\b|\bDAN\b/i, 'Danish'],
	[/\bfinnish\b|\bFIN\b/i, 'Finnish'],
	[/\bpolish\b|\bPOL\b|\bPL\b/i, 'Polish'],
	[/\bturkish\b|\bTUR\b/i, 'Turkish'],
	[/\bthai\b|\bTHA\b/i, 'Thai']
];

const SUB_PATTERNS: [RegExp, string][] = [
	[/\b(?:multi[\s.-]?)?subs?\b/i, 'Yes'],
	[/\bsubtitle[sd]?\b/i, 'Yes'],
	[/\bsrt\b/i, 'SRT'],
	[/\bhardcoded[\s.-]?subs?\b|\bhc\b/i, 'Hardcoded'],
	[/\beng[\s.-]?sub/i, 'English'],
	[/\bspa[\s.-]?sub/i, 'Spanish']
];

function extractQuality(name: string): string {
	for (const [re, label] of QUALITY_PATTERNS) {
		if (re.test(name)) {
			const source = extractSource(name);
			return source ? `${label} ${source}` : label;
		}
	}
	const source = extractSource(name);
	return source ?? 'Unknown';
}

function extractAudioQuality(name: string): string {
	const found: string[] = [];
	for (const [re, label] of AUDIO_QUALITY_PATTERNS) {
		if (re.test(name) && !found.includes(label)) {
			found.push(label);
		}
	}
	return found.length > 0 ? found.join(' ') : 'Unknown';
}

function extractSource(name: string): string | null {
	for (const [re, label] of SOURCE_PATTERNS) {
		if (re.test(name)) return label;
	}
	return null;
}

function extractLanguages(name: string): string {
	const found: string[] = [];
	for (const [re, label] of LANGUAGE_PATTERNS) {
		if (re.test(name) && !found.includes(label)) {
			found.push(label);
		}
	}
	return found.length > 0 ? found.join(', ') : 'English';
}

function extractSubs(name: string): string {
	for (const [re, label] of SUB_PATTERNS) {
		if (re.test(name)) return label;
	}
	return 'none';
}

function normalizeForMatch(s: string): string {
	return s
		.toLowerCase()
		.replace(/[^a-z0-9\s]/g, ' ')
		.replace(/\s+/g, ' ')
		.trim();
}

function extractYear(name: string): string | null {
	const match = name.match(/[\s.(](\d{4})[\s.)]/);
	return match ? match[1] : null;
}

function computeWordScore(normName: string, text: string): number {
	const words = normalizeForMatch(text)
		.split(' ')
		.filter((w) => w.length > 1);
	if (words.length === 0) return 0;
	const matched = words.filter((w) => normName.includes(w));
	return matched.length / words.length;
}

function computeRelevance(
	name: string,
	title: string,
	targetYear: string,
	artist?: string
): { relevance: number; reason: string } {
	const normName = normalizeForMatch(name);

	const torrentYear = extractYear(name);
	const yearMatches = torrentYear !== null && torrentYear === targetYear;
	const yearMismatch = torrentYear !== null && torrentYear !== targetYear;

	if (yearMismatch) {
		return { relevance: 0, reason: `year mismatch: ${torrentYear} vs ${targetYear}` };
	}

	const titleScore = computeWordScore(normName, title);

	if (artist) {
		// Music scoring: artist (40%) + album title (45%) + year (15%)
		const artistScore = computeWordScore(normName, artist);
		const yearScore = yearMatches ? 1 : 0;
		const relevance = Math.round(
			Math.min(artistScore * 40 + titleScore * 45 + yearScore * 15, 100)
		);

		const parts: string[] = [];
		if (artistScore === 1) parts.push('artist matches');
		else if (artistScore >= 0.5) parts.push('partial artist match');
		else if (artistScore > 0) parts.push('weak artist match');
		else parts.push('artist not found');

		if (titleScore === 1) parts.push('album matches');
		else if (titleScore >= 0.5) parts.push('partial album match');
		else if (titleScore > 0) parts.push('weak album match');
		else parts.push('album not found');

		if (yearMatches) parts.push('year matches');

		return { relevance, reason: parts.join(', ') };
	}

	// Video scoring (unchanged): title (85%) + year (15%)
	const yearScore = yearMatches ? 0.15 : 0;
	const relevance = Math.round(Math.min(titleScore * 85 + yearScore * 100, 100));

	const parts: string[] = [];
	if (titleScore === 1) {
		parts.push('title matches');
	} else if (titleScore >= 0.5) {
		parts.push('partial title match');
	} else {
		parts.push('weak title match');
	}
	if (yearMatches) parts.push('year matches');

	return { relevance, reason: parts.join(', ') };
}

export function parseTorrentName(
	name: string,
	targetTitle: string,
	targetYear: string,
	artist?: string
): TorrentAnalysis {
	const quality = artist ? extractAudioQuality(name) : extractQuality(name);
	const languages = extractLanguages(name);
	const subs = artist ? 'none' : extractSubs(name);
	const { relevance, reason } = computeRelevance(name, targetTitle, targetYear, artist);

	return { quality, languages, subs, relevance, reason };
}
