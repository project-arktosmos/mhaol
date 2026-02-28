import { RateLimiter } from 'common/rate-limiter';

/** Rate limiter for TMDB API (4 requests per second = 250ms between requests) */
export const tmdbRateLimiter = new RateLimiter(4, 3);
