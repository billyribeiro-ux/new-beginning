import type { LayoutLoad } from './$types';

export const prerender = false;
export const ssr = true;

export const load: LayoutLoad = () => {
	return {};
};
