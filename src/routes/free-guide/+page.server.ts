import { fail, redirect } from '@sveltejs/kit';
import type { Actions } from './$types';
import { db, schema } from '$lib/server/db/client.js';
import { leadCaptureSchema } from '$lib/utils/validators.js';
import { checkRateLimit, getClientKey } from '$lib/server/rateLimit';
import { callRust, isRustError, useRustBackend } from '$lib/server/rust/client';

// PR #6 of the BACKEND.md rollout: when `USE_RUST_BACKEND=true`, both
// actions POST to the Rust api at `/v1/public/leads`. The Drizzle path is
// kept alongside for a controlled cutover; PR #17 deletes it.

async function persistDrizzle(email: string, source: string) {
	await db.insert(schema.leads).values({
		id: `lead_${crypto.randomUUID()}`,
		email: email.toLowerCase(),
		source
	});
}

async function persistRust(
	event: Parameters<Actions[keyof Actions]>[0],
	email: string,
	source: string
) {
	await callRust('/v1/public/leads', {
		event,
		body: { email, source }
	});
}

export const actions: Actions = {
	// Free guide form on /free-guide → redirect to success page
	capture: async (event) => {
		const rl = checkRateLimit(`lead:${getClientKey(event)}`, { limit: 5, windowMs: 60_000 });
		if (!rl.ok)
			return fail(429, {
				error: `Too many requests. Try again in ${Math.ceil(rl.retryAfterMs / 1000)}s.`
			});

		const { request } = event;
		const formData = await request.formData();
		const parsed = leadCaptureSchema.safeParse({
			email: formData.get('email')?.toString() ?? '',
			source: formData.get('source')?.toString() ?? 'free-guide',
			website: formData.get('website')?.toString() ?? ''
		});
		if (!parsed.success) {
			return fail(400, {
				error: parsed.error.issues[0]?.message ?? 'Invalid submission',
				email: String(formData.get('email') ?? '')
			});
		}
		if (parsed.data.website) return fail(400, { error: 'Submission rejected' });

		try {
			if (useRustBackend()) {
				await persistRust(event, parsed.data.email, parsed.data.source ?? 'free-guide');
			} else {
				await persistDrizzle(parsed.data.email, parsed.data.source ?? 'free-guide');
			}
		} catch (e) {
			if (isRustError(e) && e.status === 429) {
				return fail(429, {
					error: `Too many requests. Try again in ${e.retry_after_secs ?? 30}s.`
				});
			}
			console.error('lead insert failed', e);
			return fail(500, { error: 'Could not save your request. Please try again.' });
		}
		redirect(303, `/free-guide/success?email=${encodeURIComponent(parsed.data.email)}`);
	},

	// Generic newsletter capture used from anywhere (home, footer) — returns success only
	subscribe: async (event) => {
		const rl = checkRateLimit(`lead:${getClientKey(event)}`, { limit: 5, windowMs: 60_000 });
		if (!rl.ok)
			return fail(429, {
				error: `Too many requests. Try again in ${Math.ceil(rl.retryAfterMs / 1000)}s.`
			});

		const { request } = event;
		const formData = await request.formData();
		const parsed = leadCaptureSchema.safeParse({
			email: formData.get('email')?.toString() ?? '',
			source: formData.get('source')?.toString() ?? 'newsletter',
			website: formData.get('website')?.toString() ?? ''
		});
		if (!parsed.success) {
			return fail(400, { error: parsed.error.issues[0]?.message ?? 'Invalid email' });
		}
		if (parsed.data.website) return fail(400, { error: 'Submission rejected' });

		try {
			if (useRustBackend()) {
				await persistRust(event, parsed.data.email, parsed.data.source ?? 'newsletter');
			} else {
				await persistDrizzle(parsed.data.email, parsed.data.source ?? 'newsletter');
			}
		} catch (e) {
			if (isRustError(e) && e.status === 429) {
				return fail(429, {
					error: `Too many requests. Try again in ${e.retry_after_secs ?? 30}s.`
				});
			}
			console.error('subscribe insert failed', e);
			return fail(500, { error: 'Could not subscribe. Please try again.' });
		}
		return { success: true };
	}
};
