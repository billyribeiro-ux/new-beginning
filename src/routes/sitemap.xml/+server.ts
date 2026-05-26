import { SITE_URL } from '$lib/utils/seo.js';
import { INDICATORS, COURSES } from '$lib/data/products.js';

export const prerender = true;

const STATIC_ROUTES: Array<{ path: string; priority: number; changefreq: string }> = [
	{ path: '/', priority: 1.0, changefreq: 'weekly' },
	{ path: '/subscription', priority: 0.95, changefreq: 'weekly' },
	{ path: '/indicators', priority: 0.9, changefreq: 'weekly' },
	{ path: '/courses', priority: 0.9, changefreq: 'weekly' },
	{ path: '/free-guide', priority: 0.8, changefreq: 'monthly' },
	{ path: '/about', priority: 0.6, changefreq: 'monthly' },
	{ path: '/contact', priority: 0.6, changefreq: 'monthly' },
	{ path: '/legal/terms', priority: 0.3, changefreq: 'yearly' },
	{ path: '/legal/privacy', priority: 0.3, changefreq: 'yearly' },
	{ path: '/legal/refund', priority: 0.3, changefreq: 'yearly' }
];

export const GET = async () => {
	const today = new Date().toISOString().slice(0, 10);

	const entries: Array<{ path: string; priority: number; changefreq: string }> = [
		...STATIC_ROUTES,
		...INDICATORS.map((p) => ({
			path: `/indicators/${p.slug}`,
			priority: 0.85,
			changefreq: 'weekly'
		})),
		...COURSES.map((p) => ({ path: `/courses/${p.slug}`, priority: 0.85, changefreq: 'weekly' }))
	];

	const xml = `<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
${entries
	.map(
		(e) => `\t<url>
\t\t<loc>${SITE_URL}${e.path}</loc>
\t\t<lastmod>${today}</lastmod>
\t\t<changefreq>${e.changefreq}</changefreq>
\t\t<priority>${e.priority.toFixed(2)}</priority>
\t</url>`
	)
	.join('\n')}
</urlset>`;

	return new Response(xml, {
		headers: { 'Content-Type': 'application/xml; charset=utf-8' }
	});
};
