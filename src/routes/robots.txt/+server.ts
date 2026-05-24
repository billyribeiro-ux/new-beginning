import { SITE_URL } from '$lib/utils/seo.js';

export const prerender = true;

export const GET = async () => {
	const body = `User-agent: *
Allow: /
Disallow: /dashboard
Disallow: /admin
Disallow: /cart
Disallow: /login
Disallow: /signup
Disallow: /forgot-password
Disallow: /reset-password

Sitemap: ${SITE_URL}/sitemap.xml
`;
	return new Response(body, {
		headers: { 'Content-Type': 'text/plain; charset=utf-8' }
	});
};
