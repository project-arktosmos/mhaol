// OMDb API response types (https://www.omdbapi.com).
// OMDb is a metadata enricher keyed by IMDb id (or title); it surfaces the
// Tomatometer (Rotten Tomatoes critic score), Metacritic, and IMDb scores
// in a single response — none of which TMDB exposes.

export interface OMDBRating {
	Source: string;
	Value: string;
}

export interface OMDBDetails {
	Title: string;
	Year: string;
	imdbID: string;
	Type: 'movie' | 'series' | 'episode' | string;
	imdbRating?: string;
	imdbVotes?: string;
	Metascore?: string;
	Ratings?: OMDBRating[];
	Response: 'True' | 'False';
	Error?: string;
}

export interface OMDBError {
	Response: 'False';
	Error: string;
}

/** Universal review shape projected out of an OMDb response. Mirrors the
 *  firkin `Review` shape persisted by the Rust backend. */
export interface NormalisedReview {
	label: string;
	score: number;
	maxScore: number;
	voteCount?: number;
}
