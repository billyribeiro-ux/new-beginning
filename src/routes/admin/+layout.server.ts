import { error } from '@sveltejs/kit';
import type { LayoutServerLoad } from './$types';

export const load: LayoutServerLoad = ({ locals }) => {
	if (locals.user?.role !== 'admin') {
		error(403, { message: 'Admin access required.', code: 'forbidden' });
	}
	return { user: locals.user };
};
