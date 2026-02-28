import { RateLimiter } from 'common/rate-limiter';

/** Rate limiter for MusicBrainz API (0.8 requests per second = 1250ms between requests) */
export const musicbrainzRateLimiter = new RateLimiter(0.8, 3);
