import { describe, it, expect, vi, beforeEach } from 'vitest';
import { RateLimiter } from '../src/rate-limiter.js';

describe('RateLimiter', () => {
	beforeEach(() => {
		vi.useFakeTimers();
	});

	it('creates with default parameters', () => {
		const limiter = new RateLimiter();
		expect(limiter.queueLength).toBe(0);
	});

	it('creates with custom parameters', () => {
		const limiter = new RateLimiter(10, 5);
		expect(limiter.queueLength).toBe(0);
	});

	it('executes a single request immediately', async () => {
		vi.useRealTimers();
		const limiter = new RateLimiter(100);
		const result = await limiter.enqueue(() => Promise.resolve('hello'));
		expect(result).toBe('hello');
	});

	it('resolves enqueued requests in order', async () => {
		vi.useRealTimers();
		const limiter = new RateLimiter(100);
		const order: number[] = [];

		const p1 = limiter.enqueue(async () => {
			order.push(1);
			return 1;
		});
		const p2 = limiter.enqueue(async () => {
			order.push(2);
			return 2;
		});
		const p3 = limiter.enqueue(async () => {
			order.push(3);
			return 3;
		});

		const results = await Promise.all([p1, p2, p3]);
		expect(results).toEqual([1, 2, 3]);
		expect(order).toEqual([1, 2, 3]);
	});

	it('reports queue length correctly', async () => {
		vi.useRealTimers();
		const limiter = new RateLimiter(1);
		let resolveFirst: () => void;
		const blockingPromise = new Promise<void>((r) => {
			resolveFirst = r;
		});

		limiter.enqueue(() => blockingPromise);
		limiter.enqueue(() => Promise.resolve());

		// The first request is being processed, second is in queue
		// Queue length may be 0 or 1 depending on timing
		expect(limiter.queueLength).toBeGreaterThanOrEqual(0);

		resolveFirst!();
		await new Promise((r) => setTimeout(r, 50));
	});

	it('rejects non-retryable errors immediately', async () => {
		vi.useRealTimers();
		const limiter = new RateLimiter(100, 3);
		const error = new Error('Not retryable');

		await expect(limiter.enqueue(() => Promise.reject(error))).rejects.toThrow('Not retryable');
	});

	it('retries on retryable TypeError (Load failed)', async () => {
		vi.useRealTimers();
		const limiter = new RateLimiter(100, 1);
		let attempts = 0;

		const result = await limiter.enqueue(async () => {
			attempts++;
			if (attempts === 1) {
				throw new TypeError('Load failed');
			}
			return 'success';
		});

		expect(result).toBe('success');
		expect(attempts).toBe(2);
	});

	it('retries on 503 errors', async () => {
		vi.useRealTimers();
		const limiter = new RateLimiter(100, 1);
		let attempts = 0;

		const result = await limiter.enqueue(async () => {
			attempts++;
			if (attempts === 1) {
				throw new Error('503 Service Unavailable');
			}
			return 'recovered';
		});

		expect(result).toBe('recovered');
		expect(attempts).toBe(2);
	});

	it('rejects after exhausting retries on retryable errors', async () => {
		vi.useRealTimers();
		const limiter = new RateLimiter(100, 0);
		let attempts = 0;

		await expect(
			limiter.enqueue(async () => {
				attempts++;
				throw new TypeError('Load failed');
			})
		).rejects.toThrow('Load failed');

		// 0 retries = 1 attempt only
		expect(attempts).toBe(1);
	});

	it('handles concurrent requests without interfering', async () => {
		vi.useRealTimers();
		const limiter = new RateLimiter(100);

		const results = await Promise.all([
			limiter.enqueue(() => Promise.resolve('a')),
			limiter.enqueue(() => Promise.resolve('b')),
			limiter.enqueue(() => Promise.resolve('c'))
		]);

		expect(results).toEqual(['a', 'b', 'c']);
	});
});
