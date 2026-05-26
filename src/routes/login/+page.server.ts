import { fail, redirect } from '@sveltejs/kit';
import type { Actions } from './$types';
import { loginSchema } from '$lib/utils/validators.js';

export const actions: Actions = {
	default: async ({ request, cookies }) => {
		const formData = await request.formData();
		const parsed = loginSchema.safeParse({
			email: formData.get('email')?.toString() ?? '',
			password: formData.get('password')?.toString() ?? '',
			remember: formData.get('remember')?.toString() === 'on'
		});
		if (!parsed.success) {
			return fail(400, { error: parsed.error.issues[0]?.message ?? 'Invalid login.' });
		}

		// Phase 1: stub. Set mock cookie so dashboard can render.
		cookies.set('tradeflex-mock-user', 'member', {
			path: '/',
			maxAge: parsed.data.remember ? 60 * 60 * 24 * 30 : 60 * 60 * 24
		});

		redirect(303, '/dashboard');
	}
};
