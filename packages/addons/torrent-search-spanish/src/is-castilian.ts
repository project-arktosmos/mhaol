/**
 * Castilian Spanish (España) language detection.
 *
 * Latin American Spanish (`latino`, `latam`, `es-MX`, etc.) is rejected:
 * the latino marker wins even when a Castilian marker is also present.
 */

const SPANISH_TOKENS = [
	'castellano',
	'castelhano',
	'español',
	'espanol',
	'spanish',
	'spa',
	'esp',
	'cast',
	'es-es'
];

const LATINO_TOKENS = [
	'latino',
	'latina',
	'latinoamericano',
	'latinoamericana',
	'latinoamerica',
	'latam',
	'lat',
	'es-mx',
	'es-la',
	'es-ar',
	'es-cl',
	'es-419',
	'mexicano',
	'mexicana'
];

function isWordChar(ch: string): boolean {
	return /[a-z0-9]/.test(ch);
}

function containsToken(lower: string, tokens: readonly string[]): boolean {
	for (const token of tokens) {
		const tlen = token.length;
		let start = 0;
		while (true) {
			const idx = lower.indexOf(token, start);
			if (idx === -1) break;
			const prevOk = idx === 0 || !isWordChar(lower.charAt(idx - 1));
			const end = idx + tlen;
			const nextOk = end === lower.length || !isWordChar(lower.charAt(end));
			if (prevOk && nextOk) return true;
			start = idx + 1;
		}
	}
	return false;
}

/** True when `name` is a Castilian Spanish release (and not Latin American). */
export function isCastilianRelease(name: string): boolean {
	const lower = name.toLowerCase();
	if (containsToken(lower, LATINO_TOKENS)) return false;
	return containsToken(lower, SPANISH_TOKENS);
}
