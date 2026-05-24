import { fail, redirect } from '@sveltejs/kit';
import type { Actions } from './$types';
import { db, schema } from '$lib/server/db/client.js';
import { leadCaptureSchema } from '$lib/utils/validators.js';

function randomId() {
	return 'lead_' + Math.random().toString(36).slice(2, 10) + Date.now().toString(36);
}

async function persist(email: string, source: string) {
	await db.insert(schema.leads).values({
		id: randomId(),
		email: email.toLowerCase(),
		source
	});
}

export const actions: Actions = {
	// Free guide form on /free-guide → redirect to success page
	capture: async ({ request }) => {
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
			await persist(parsed.data.email, parsed.data.source ?? 'free-guide');
		} catch (e) {
			console.error('lead insert failed', e);
			return fail(500, { error: 'Could not save your request. Please try again.' });
		}
		redirect(303, `/free-guide/success?email=${encodeURIComponent(parsed.data.email)}`);
	},

	// Generic newsletter capture used from anywhere (home, footer) — returns success only
	subscribe: async ({ request }) => {
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
			await persist(parsed.data.email, parsed.data.source ?? 'newsletter');
		} catch (e) {
			console.error('subscribe insert failed', e);
			return fail(500, { error: 'Could not subscribe. Please try again.' });
		}
		return { success: true };
	}
};
