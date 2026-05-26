import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';
import { getProductBySlug } from '$lib/data/products.js';

export const load: PageLoad = ({ params }) => {
	const course = getProductBySlug(params.slug);
	if (!course || course.kind !== 'course') error(404, 'Course not found');
	return { course };
};
