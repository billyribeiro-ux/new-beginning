// src/lib/server/rust/client.ts
//
// BACKEND.md §20: the BFF talks to the Rust api via `fetch` with
//   - X-Service-Token (env-loaded, never sent to the browser)
//   - X-Request-Id    (propagated back to the browser on the response)
//   - Cookie          (the user's session cookie, forwarded both ways)
//
// PR #6 puts this helper in place and uses it from the `/free-guide` and
// `/contact` actions when `USE_RUST_BACKEND === 'true'`. PR #17 flips the
// flag globally and deletes the Drizzle paths.

import { env } from '$env/dynamic/private';
import type { RequestEvent } from '@sveltejs/kit';

export interface RustError {
	status: number;
	code: string;
	message: string;
	request_id: string;
	retry_after_secs?: number;
}

const BASE_URL = env.RUST_API_BASE_URL ?? 'http://127.0.0.1:8081';
const SERVICE_TOKEN = env.SERVICE_TOKEN ?? '';

// PR #17: cutover. The flag flips from opt-in to opt-out — the Rust
// backend is the default. Setting USE_RUST_BACKEND=false explicitly
// falls back to the legacy Drizzle paths (kept around for a single
// rollback window; deletion lands once the cutover is observed clean).
export function useRustBackend(): boolean {
	return env.USE_RUST_BACKEND !== 'false';
}

function newRequestId(): string {
	// Browsers don't have crypto.randomUUID() v7 yet; v4 is fine for an id.
	return (
		globalThis.crypto?.randomUUID?.() ?? `req-${Date.now()}-${Math.random().toString(36).slice(2)}`
	);
}

interface RustCallOptions {
	method?: 'GET' | 'POST' | 'PATCH' | 'PUT' | 'DELETE';
	body?: unknown;
	/** Pass the incoming RequestEvent to forward the user's cookie + IP. */
	event?: RequestEvent;
	/** Override the generated request id (e.g. when the upstream provided one). */
	requestId?: string;
}

export async function callRust<T>(path: string, opts: RustCallOptions = {}): Promise<T> {
	if (!SERVICE_TOKEN) {
		throw new Error('SERVICE_TOKEN env var is unset; BFF cannot reach Rust');
	}

	const requestId = opts.requestId ?? newRequestId();
	const headers: Record<string, string> = {
		'x-service-token': SERVICE_TOKEN,
		'x-request-id': requestId
	};
	if (opts.body !== undefined) {
		headers['content-type'] = 'application/json';
	}
	const incomingCookie = opts.event?.request.headers.get('cookie');
	if (incomingCookie) {
		headers['cookie'] = incomingCookie;
	}
	// Forward client IP for the limiter buckets that key on it. Rust reads
	// the peer address from the TCP socket, so a direct `fetch` from the BFF
	// box would attribute all requests to the BFF's IP. The `x-forwarded-for`
	// header makes it trivial to swap in a TrustedProxy layer on the Rust
	// side later; for PR #6 it's informational.
	const clientIp = opts.event?.getClientAddress?.();
	if (clientIp) {
		headers['x-forwarded-for'] = clientIp;
	}

	const url = `${BASE_URL.replace(/\/$/, '')}${path}`;
	const res = await fetch(url, {
		method: opts.method ?? 'POST',
		headers,
		body: opts.body !== undefined ? JSON.stringify(opts.body) : undefined
	});

	if (!res.ok) {
		let parsed: { error?: { code?: string; message?: string }; request_id?: string } | null = null;
		try {
			parsed = await res.json();
		} catch {
			/* non-JSON body — fall through to defaults */
		}
		const retry = res.headers.get('retry-after');
		const err: RustError = {
			status: res.status,
			code: parsed?.error?.code ?? 'upstream_error',
			message: parsed?.error?.message ?? res.statusText,
			request_id: parsed?.request_id ?? requestId,
			retry_after_secs: retry ? Number(retry) || undefined : undefined
		};
		throw err;
	}

	// 204 No Content — return an empty object so callers don't need to
	// special-case.
	if (res.status === 204) return {} as T;
	return (await res.json()) as T;
}

export function isRustError(e: unknown): e is RustError {
	return typeof e === 'object' && e !== null && 'status' in e && 'code' in e && 'message' in e;
}
