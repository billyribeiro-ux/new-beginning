import type { RequestEvent } from '@sveltejs/kit';

/**
 * In-memory sliding-window rate limiter.
 *
 * Server-only. Suitable for single-instance deployments (or as a cheap first
 * line of defence in front of a real distributed limiter). For multi-instance
 * production, swap the backing Map for Redis.
 */

type RateLimitOk = { ok: true };
type RateLimitDenied = { ok: false; retryAfterMs: number };
export type RateLimitResult = RateLimitOk | RateLimitDenied;

interface RateLimitOptions {
	limit: number;
	windowMs: number;
}

const buckets = new Map<string, number[]>();

export function checkRateLimit(key: string, opts: RateLimitOptions): RateLimitResult {
	const now = Date.now();
	const cutoff = now - opts.windowMs;

	const existing = buckets.get(key);
	const timestamps = existing ? existing.filter((t) => t > cutoff) : [];

	if (timestamps.length >= opts.limit) {
		const oldest = timestamps[0] ?? now;
		const retryAfterMs = Math.max(0, oldest + opts.windowMs - now);
		// Persist the pruned list so memory doesn't grow unbounded on rejected calls.
		buckets.set(key, timestamps);
		return { ok: false, retryAfterMs };
	}

	timestamps.push(now);
	buckets.set(key, timestamps);
	return { ok: true };
}

/**
 * Best-effort client identifier. SvelteKit can throw from
 * `getClientAddress()` when no adapter has populated it (e.g. some test
 * harnesses), so we swallow the error and degrade to a shared bucket rather
 * than crashing the request.
 */
export function getClientKey(event: RequestEvent): string {
	try {
		return event.getClientAddress();
	} catch {
		return 'unknown';
	}
}

// --- Periodic cleanup -------------------------------------------------------

const CLEANUP_INTERVAL_MS = 10 * 60 * 1000;

declare global {
	// eslint-disable-next-line no-var
	var __rateLimitCleanupStarted: boolean | undefined;
}

if (typeof globalThis !== 'undefined' && !globalThis.__rateLimitCleanupStarted) {
	globalThis.__rateLimitCleanupStarted = true;
	const interval = setInterval(() => {
		const now = Date.now();
		// We don't know each key's windowMs at cleanup time, so we use a generous
		// upper bound: drop entries whose newest timestamp is older than an hour.
		// Active keys are pruned by checkRateLimit on every call; this loop only
		// reclaims memory for keys that have gone quiet.
		const staleCutoff = now - 60 * 60 * 1000;
		for (const [key, timestamps] of buckets) {
			if (timestamps.length === 0) {
				buckets.delete(key);
				continue;
			}
			const newest = timestamps[timestamps.length - 1] ?? 0;
			if (newest < staleCutoff) {
				buckets.delete(key);
			}
		}
	}, CLEANUP_INTERVAL_MS);
	// Don't keep the event loop alive purely for cleanup (Node only).
	if (typeof (interval as { unref?: () => void }).unref === 'function') {
		(interval as { unref: () => void }).unref();
	}
}
