import { fail, redirect } from '@sveltejs/kit';
import type { Actions } from './$types';
import { signupSchema } from '$lib/utils/validators.js';

export const actions: Actions = {
	default: async ({ request, cookies }) => {
		const formData = await request.formData();
		const parsed = signupSchema.safeParse({
			name: formData.get('name')?.toString() ?? '',
			email: formData.get('email')?.toString() ?? '',
			password: formData.get('password')?.toString() ?? '',
			confirm: formData.get('confirm')?.toString() ?? '',
			terms: formData.get('terms')?.toString() === 'on'
		});
		if (!parsed.success) {
			return fail(400, { error: parsed.error.issues[0]?.message ?? 'Please review the form.' });
		}
		// Phase 1: stub. Cookie-based "session".
		cookies.set('tradeflex-mock-user', 'member', { path: '/', maxAge: 60 * 60 * 24 * 30 });
		redirect(303, '/dashboard');
	}
};
