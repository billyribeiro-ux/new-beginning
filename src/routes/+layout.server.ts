import type { LayoutServerLoad } from './$types';

export const load: LayoutServerLoad = ({ locals, url }) => {
	return {
		user: locals.user,
		pathname: url.pathname
	};
};
