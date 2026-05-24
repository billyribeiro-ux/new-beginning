import type { PageLoad } from './$types';

export const load: PageLoad = ({ url }) => {
	return {
		email: url.searchParams.get('email') ?? ''
	};
};
