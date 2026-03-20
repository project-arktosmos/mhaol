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

function computeRelevance(name: string, title: string, targetYear: string): { relevance: number; reason: string } {
	const normName = normalizeForMatch(name);
	const normTitle = normalizeForMatch(title);
	const titleWords = normTitle.split(' ').filter((w) => w.length > 1);

	const matchedWords = titleWords.filter((w) => normName.includes(w));
	const wordScore = titleWords.length > 0 ? matchedWords.length / titleWords.length : 0;

	const torrentYear = extractYear(name);
	const yearMatches = torrentYear !== null && torrentYear === targetYear;
	const yearMismatch = torrentYear !== null && torrentYear !== targetYear;

	if (yearMismatch) {
		return { relevance: 0, reason: `year mismatch: ${torrentYear} vs ${targetYear}` };
	}

	const yearScore = yearMatches ? 0.15 : 0;
	const relevance = Math.round(Math.min((wordScore * 85 + yearScore * 100), 100));

	const parts: string[] = [];
	if (wordScore === 1) {
		parts.push('title matches');
	} else if (wordScore >= 0.5) {
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
	targetYear: string
): TorrentAnalysis {
	const quality = extractQuality(name);
	const languages = extractLanguages(name);
	const subs = extractSubs(name);
	const { relevance, reason } = computeRelevance(name, targetTitle, targetYear);

	return { quality, languages, subs, relevance, reason };
}
