import { fail } from '@sveltejs/kit';
import type { Actions } from './$types';
import { db, schema } from '$lib/server/db/client.js';
import { contactSchema } from '$lib/utils/validators.js';

function randomId() {
	return 'msg_' + Math.random().toString(36).slice(2, 10) + Date.now().toString(36);
}

export const actions: Actions = {
	default: async ({ request }) => {
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
			await db.insert(schema.contactMessages).values({
				id: randomId(),
				name: parsed.data.name,
				email: parsed.data.email.toLowerCase(),
				subject: parsed.data.subject,
				body: parsed.data.body
			});
		} catch (e) {
			console.error('contact insert failed', e);
			return fail(500, { error: 'Could not send your message. Please try again.' });
		}

		return { success: true };
	}
};
