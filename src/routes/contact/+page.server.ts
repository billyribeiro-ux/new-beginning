import { fail } from '@sveltejs/kit';
import type { Actions } from './$types';
import { db, schema } from '$lib/server/db/client.js';
import { contactSchema } from '$lib/utils/validators.js';
import { checkRateLimit, getClientKey } from '$lib/server/rateLimit';
import { callRust, isRustError, useRustBackend } from '$lib/server/rust/client';

// PR #6: when `USE_RUST_BACKEND=true`, the contact form POSTs to
// `/v1/public/contact` instead of writing through Drizzle.

export const actions: Actions = {
	default: async (event) => {
		const rl = checkRateLimit(`contact:${getClientKey(event)}`, {
			limit: 3,
			windowMs: 60_000
		});
		if (!rl.ok)
			return fail(429, {
				error: `Too many requests. Try again in ${Math.ceil(rl.retryAfterMs / 1000)}s.`
			});

		const { request } = event;
		const formData = await request.formData();
		const data = {
			name: formData.get('name')?.toString() ?? '',
			email: formData.get('email')?.toString() ?? '',
			subject: formData.get('subject')?.toString() ?? '',
			body: formData.get('body')?.toString() ?? '',
			website: formData.get('website')?.toString() ?? ''
		};
		const parsed = contactSchema.safeParse(data);
		if (!parsed.success) {
			return fail(400, {
				error: parsed.error.issues[0]?.message ?? 'Please review the form.',
				values: { name: data.name, email: data.email, subject: data.subject, body: data.body }
			});
		}
		if (parsed.data.website) return fail(400, { error: 'Submission rejected' });

		try {
			if (useRustBackend()) {
				await callRust('/v1/public/contact', {
					event,
					body: {
						name: parsed.data.name,
						email: parsed.data.email,
						subject: parsed.data.subject,
						body: parsed.data.body
					}
				});
			} else {
				await db.insert(schema.contactMessages).values({
					id: `msg_${crypto.randomUUID()}`,
					name: parsed.data.name,
					email: parsed.data.email.toLowerCase(),
					subject: parsed.data.subject,
					body: parsed.data.body
				});
			}
		} catch (e) {
			if (isRustError(e) && e.status === 429) {
				return fail(429, {
					error: `Too many requests. Try again in ${e.retry_after_secs ?? 30}s.`
				});
			}
			console.error('contact insert failed', e);
			return fail(500, { error: 'Could not send your message. Please try again.' });
		}

		return { success: true };
	}
};
