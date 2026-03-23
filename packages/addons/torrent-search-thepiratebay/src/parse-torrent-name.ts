import type { TorrentAnalysis } from './types.js';

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

const CONSOLE_PATTERNS: [RegExp, string][] = [
	[/\bGBA\b/i, 'Game Boy Advance'],
	[/\bGame[\s.-]?Boy[\s.-]?Advance\b/i, 'Game Boy Advance'],
	[/\bGBC\b/i, 'Game Boy Color'],
	[/\bGame[\s.-]?Boy[\s.-]?Color\b/i, 'Game Boy Color'],
	[/\bGB\b/, 'Game Boy'],
	[/\bGame[\s.-]?Boy\b/i, 'Game Boy'],
	[/\bNES\b/i, 'NES/Famicom'],
	[/\bFamicom\b/i, 'NES/Famicom'],
	[/\bSNES\b/i, 'SNES/Super Famicom'],
	[/\bSuper[\s.-]?Nintendo\b/i, 'SNES/Super Famicom'],
	[/\bSuper[\s.-]?Famicom\b/i, 'SNES/Super Famicom'],
	[/\bSFC\b/i, 'SNES/Super Famicom'],
	[/\bN64\b/i, 'Nintendo 64'],
	[/\bNintendo[\s.-]?64\b/i, 'Nintendo 64'],
	[/\bNDS\b/i, 'Nintendo DS'],
	[/\bNintendo[\s.-]?DS\b/i, 'Nintendo DS'],
	[/\bGameCube\b/i, 'GameCube'],
	[/\bGCN\b/i, 'GameCube'],
	[/\bGenesis\b/i, 'Genesis/Mega Drive'],
	[/\bMega[\s.-]?Drive\b/i, 'Genesis/Mega Drive'],
	[/\bMaster[\s.-]?System\b/i, 'Master System'],
	[/\bSMS\b/i, 'Master System'],
	[/\bPS1\b|\bPSX\b|\bPlayStation[\s.-]?1?\b/i, 'PlayStation'],
	[/\bPS2\b|\bPlayStation[\s.-]?2\b/i, 'PlayStation 2'],
	[/\bPSP\b/i, 'PSP'],
	[/\bNeo[\s.-]?Geo[\s.-]?Pocket\b/i, 'Neo Geo Pocket'],
	[/\bNGP\b/i, 'Neo Geo Pocket'],
	[/\bLynx\b/i, 'Atari Lynx'],
	[/\bJaguar\b/i, 'Atari Jaguar'],
	[/\bPC[\s.-]?Engine\b|\bTurboGrafx\b/i, 'PC Engine/TurboGrafx-16']
];

const ROM_FORMAT_PATTERNS: [RegExp, string][] = [
	[/\.iso\b/i, 'ISO'],
	[/\.bin\b/i, 'BIN'],
	[/\.cue\b/i, 'BIN/CUE'],
	[/\.nds\b/i, 'NDS'],
	[/\.gba\b/i, 'GBA'],
	[/\.gbc\b/i, 'GBC'],
	[/\.gb\b/i, 'GB'],
	[/\.nes\b/i, 'NES'],
	[/\.sfc\b|\bsmc\b/i, 'SFC'],
	[/\.n64\b|\b\.z64\b|\b\.v64\b/i, 'N64'],
	[/\.zip\b/i, 'ZIP'],
	[/\.7z\b/i, '7Z'],
	[/\.rar\b/i, 'RAR']
];

function extractRomFormat(name: string): string {
	for (const [re, label] of ROM_FORMAT_PATTERNS) {
		if (re.test(name)) return label;
	}
	return 'Unknown';
}

function extractConsole(name: string): string | null {
	for (const [re, label] of CONSOLE_PATTERNS) {
		if (re.test(name)) return label;
	}
	return null;
}

export function extractSeasonEpisode(name: string): {
	season: number | null;
	episode: number | null;
	isCompleteSeries: boolean;
} {
	// Complete series patterns
	if (
		/\bcomplete[\s.-]?series\b/i.test(name) ||
		/\bfull[\s.-]?series\b/i.test(name) ||
		/\ball[\s.-]?seasons?\b/i.test(name) ||
		/\bcomplete\b.*\bseason\s*1\s*-\s*\d+/i.test(name) ||
		/\bs0?1[\s.-]?-[\s.-]?s?\d+/i.test(name)
	) {
		return { season: null, episode: null, isCompleteSeries: true };
	}

	// S01E01 pattern
	const seMatch = name.match(/\bS(\d{1,2})[\s.-]?E(\d{1,3})\b/i);
	if (seMatch) {
		return {
			season: parseInt(seMatch[1], 10),
			episode: parseInt(seMatch[2], 10),
			isCompleteSeries: false
		};
	}

	// Season + Episode spelled out: "Season 1 Episode 5"
	const speltMatch = name.match(/\bSeason[\s.-]?(\d{1,2})[\s.-]?Episode[\s.-]?(\d{1,3})\b/i);
	if (speltMatch) {
		return {
			season: parseInt(speltMatch[1], 10),
			episode: parseInt(speltMatch[2], 10),
			isCompleteSeries: false
		};
	}

	// S01 only (season pack, no episode)
	const sOnly = name.match(/\bS(\d{1,2})\b(?![\s.-]?E\d)/i);
	if (sOnly) {
		return { season: parseInt(sOnly[1], 10), episode: null, isCompleteSeries: false };
	}

	// "Season 1" / "Season.2" (season pack)
	const seasonSpelt = name.match(/\bSeason[\s.-]?(\d{1,2})\b(?![\s.-]?Episode)/i);
	if (seasonSpelt) {
		return { season: parseInt(seasonSpelt[1], 10), episode: null, isCompleteSeries: false };
	}

	return { season: null, episode: null, isCompleteSeries: false };
}

function computeRelevance(
	name: string,
	title: string,
	targetYear: string,
	artist?: string,
	consoleName?: string
): { relevance: number; reason: string } {
	const normName = normalizeForMatch(name);

	const torrentYear = extractYear(name);
	const yearMatches = torrentYear !== null && torrentYear === targetYear;
	const yearMismatch = torrentYear !== null && torrentYear !== targetYear;

	if (yearMismatch) {
		return { relevance: 0, reason: `year mismatch: ${torrentYear} vs ${targetYear}` };
	}

	const titleScore = computeWordScore(normName, title);

	if (consoleName) {
		// Game scoring: title (70%) + console match (15%) + year (15%)
		const detectedConsole = extractConsole(name);
		const consoleMatches =
			detectedConsole !== null &&
			normalizeForMatch(detectedConsole).includes(normalizeForMatch(consoleName));
		const consoleScore = consoleMatches ? 1 : 0;
		const yearScore = yearMatches ? 1 : 0;
		const relevance = Math.round(
			Math.min(titleScore * 70 + consoleScore * 15 + yearScore * 15, 100)
		);

		const parts: string[] = [];
		if (titleScore === 1) parts.push('title matches');
		else if (titleScore >= 0.5) parts.push('partial title match');
		else if (titleScore > 0) parts.push('weak title match');
		else parts.push('title not found');

		if (consoleMatches) parts.push('console matches');
		else if (detectedConsole) parts.push(`wrong console: ${detectedConsole}`);

		if (yearMatches) parts.push('year matches');

		return { relevance, reason: parts.join(', ') };
	}

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
	artist?: string,
	consoleName?: string
): TorrentAnalysis {
	let quality: string;
	let languages: string;
	let subs: string;

	if (consoleName) {
		quality = extractRomFormat(name);
		languages = 'N/A';
		subs = 'none';
	} else if (artist) {
		quality = extractAudioQuality(name);
		languages = extractLanguages(name);
		subs = 'none';
	} else {
		quality = extractQuality(name);
		languages = extractLanguages(name);
		subs = extractSubs(name);
	}

	const { relevance, reason } = computeRelevance(
		name,
		targetTitle,
		targetYear,
		artist,
		consoleName
	);

	// Extract season/episode info for video content (not music/games)
	const isVideo = !artist && !consoleName;
	const se = isVideo ? extractSeasonEpisode(name) : null;

	return {
		quality,
		languages,
		subs,
		relevance,
		reason,
		seasonNumber: se?.season ?? null,
		episodeNumber: se?.episode ?? null,
		isCompleteSeries: se?.isCompleteSeries ?? false
	};
}
