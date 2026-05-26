import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';
import { getAllProducts } from '$lib/data/products.js';

export const load: PageLoad = ({ params }) => {
	const product = getAllProducts().find((p) => p.id === params.id);
	if (!product) error(404, 'Product not found');
	return { product };
};
