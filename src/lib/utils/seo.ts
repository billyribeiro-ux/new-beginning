import { env } from '$env/dynamic/public';

export const SITE_NAME = 'TradeFlex Trading';
export const SITE_TAGLINE = 'Precision day-trading systems, indicators, and education.';
export const SITE_URL = env.PUBLIC_SITE_URL?.replace(/\/$/, '') || 'https://tradeflextrading.com';

export const TITLE_TEMPLATE = (page: string) =>
	page ? `${page} · ${SITE_NAME}` : `${SITE_NAME} — ${SITE_TAGLINE}`;

export function absoluteUrl(pathname: string): string {
	if (pathname.startsWith('http')) return pathname;
	return `${SITE_URL}${pathname.startsWith('/') ? '' : '/'}${pathname}`;
}

export function defaultOgImage(): string {
	return absoluteUrl('/og/default.png');
}

export type SeoMeta = {
	title?: string;
	description?: string;
	canonical?: string;
	image?: string;
	noindex?: boolean;
	keywords?: string[];
	type?: 'website' | 'article' | 'product';
	publishedAt?: string;
	updatedAt?: string;
	author?: string;
};
