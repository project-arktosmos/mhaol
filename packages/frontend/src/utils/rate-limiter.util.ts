/**
 * Queue-based rate limiter for API calls.
 * Ensures requests are spaced by a minimum interval with retry support.
 */

type QueuedRequest<T> = {
	fn: () => Promise<T>;
	resolve: (value: T) => void;
	reject: (error: unknown) => void;
	retries: number;
};

export class RateLimiter {
	private queue: QueuedRequest<unknown>[] = [];
	private processing = false;
	private lastRequestTime = 0;
	private minInterval: number;
	private maxRetries: number;

	constructor(requestsPerSecond: number = 4, maxRetries: number = 3) {
		this.minInterval = 1000 / requestsPerSecond;
		this.maxRetries = maxRetries;
	}

	async enqueue<T>(fn: () => Promise<T>): Promise<T> {
		return new Promise((resolve, reject) => {
			this.queue.push({
				fn,
				resolve: resolve as (value: unknown) => void,
				reject,
				retries: 0
			});
			this.process();
		});
	}

	get queueLength(): number {
		return this.queue.length;
	}

	get isProcessing(): boolean {
		return this.processing;
	}

	private async process(): Promise<void> {
		if (this.processing) return;
		this.processing = true;

		while (this.queue.length > 0) {
			const now = Date.now();
			const timeSinceLastRequest = now - this.lastRequestTime;

			if (timeSinceLastRequest < this.minInterval) {
				await this.sleep(this.minInterval - timeSinceLastRequest);
			}

			const request = this.queue.shift();
			if (request) {
				this.lastRequestTime = Date.now();
				try {
					const result = await request.fn();
					request.resolve(result);
				} catch (error) {
					if (request.retries < this.maxRetries && this.isRetryableError(error)) {
						request.retries++;
						const backoffDelay = Math.pow(2, request.retries) * 1000;
						await this.sleep(backoffDelay);
						this.queue.unshift(request);
					} else {
						request.reject(error);
					}
				}
			}
		}

		this.processing = false;
	}

	private isRetryableError(error: unknown): boolean {
		if (error instanceof TypeError && error.message.includes('Load failed')) {
			return true;
		}
		if (error instanceof Error && error.message.includes('503')) {
			return true;
		}
		return false;
	}

	private sleep(ms: number): Promise<void> {
		return new Promise((resolve) => setTimeout(resolve, ms));
	}
}

/** Rate limiter for MusicBrainz API (0.8 requests per second = 1250ms between requests) */
export const musicBrainzRateLimiter = new RateLimiter(0.8, 3);
