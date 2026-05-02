import type { NormalisedReview, OMDBDetails, OMDBRating } from './types.js';

/** Parse an OMDb `Ratings` value (e.g. `"92%"`, `"7.8/10"`, `"78/100"`)
 *  into `{ score, maxScore }`. Returns `null` for unparseable strings so
 *  callers can drop the entry rather than persist a bogus score. */
export function parseRatingValue(value: string): { score: number; maxScore: number } | null {
	const v = value.trim();

	const pct = v.match(/^(\d+(?:\.\d+)?)\s*%$/);
	if (pct) {
		const score = Number(pct[1]);
		if (Number.isFinite(score)) return { score, maxScore: 100 };
	}

	const frac = v.match(/^(\d+(?:\.\d+)?)\s*\/\s*(\d+(?:\.\d+)?)$/);
	if (frac) {
		const score = Number(frac[1]);
		const maxScore = Number(frac[2]);
		if (Number.isFinite(score) && Number.isFinite(maxScore) && maxScore > 0) {
			return { score, maxScore };
		}
	}

	return null;
}

/** Map OMDb's free-form `Source` strings to the canonical labels we
 *  persist on firkins. Anything we don't know about is passed through
 *  verbatim so future OMDb additions still surface, just unbranded. */
function canonicaliseSource(source: string): string {
	const s = source.trim().toLowerCase();
	if (s === 'internet movie database' || s === 'imdb') return 'IMDb';
	if (s === 'rotten tomatoes') return 'Rotten Tomatoes';
	if (s === 'metacritic') return 'Metacritic';
	return source.trim();
}

function ratingToReview(r: OMDBRating): NormalisedReview | null {
	const parsed = parseRatingValue(r.Value);
	if (!parsed) return null;
	return {
		label: canonicaliseSource(r.Source),
		score: parsed.score,
		maxScore: parsed.maxScore
	};
}

/** Project an OMDb response into the firkin `Review` shape. Combines the
 *  `Ratings` array (Rotten Tomatoes, Metacritic, IMDb-as-x/10) with the
 *  `imdbVotes` count when present so the IMDb entry carries a `voteCount`.
 *  Duplicate labels are collapsed (the entry that survives is the one
 *  carrying a `voteCount`, otherwise the first occurrence). */
export function reviewsFromOmdb(payload: OMDBDetails): NormalisedReview[] {
	const out: NormalisedReview[] = [];

	for (const r of payload.Ratings ?? []) {
		const review = ratingToReview(r);
		if (review) out.push(review);
	}

	const imdbVotes = parseImdbVotes(payload.imdbVotes);
	if (imdbVotes !== null) {
		const idx = out.findIndex((r) => r.label === 'IMDb');
		if (idx >= 0) out[idx] = { ...out[idx], voteCount: imdbVotes };
	}

	return out;
}

function parseImdbVotes(raw: string | undefined): number | null {
	if (!raw) return null;
	const cleaned = raw.replace(/[,\s]/g, '');
	if (!/^\d+$/.test(cleaned)) return null;
	const n = Number(cleaned);
	return Number.isFinite(n) ? n : null;
}
