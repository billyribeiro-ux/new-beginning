import { SITE_NAME, SITE_URL, absoluteUrl } from './seo.js';

type JsonLd = Record<string, unknown>;

export function organizationLd(): JsonLd {
	return {
		'@context': 'https://schema.org',
		'@type': 'Organization',
		'@id': `${SITE_URL}/#organization`,
		name: SITE_NAME,
		legalName: 'TradeFlex Trading LLC',
		url: SITE_URL,
		logo: absoluteUrl('/og/logo.png'),
		sameAs: [
			'https://x.com/tradeflextrading',
			'https://www.youtube.com/@tradeflextrading',
			'https://www.linkedin.com/company/tradeflextrading'
		],
		contactPoint: [
			{
				'@type': 'ContactPoint',
				email: 'hello@tradeflextrading.com',
				contactType: 'customer support',
				availableLanguage: ['English']
			}
		]
	};
}

export function websiteLd(): JsonLd {
	return {
		'@context': 'https://schema.org',
		'@type': 'WebSite',
		'@id': `${SITE_URL}/#website`,
		name: SITE_NAME,
		url: SITE_URL,
		potentialAction: {
			'@type': 'SearchAction',
			target: `${SITE_URL}/search?q={search_term_string}`,
			'query-input': 'required name=search_term_string'
		},
		publisher: { '@id': `${SITE_URL}/#organization` }
	};
}

export function breadcrumbLd(items: Array<{ name: string; url: string }>): JsonLd {
	return {
		'@context': 'https://schema.org',
		'@type': 'BreadcrumbList',
		itemListElement: items.map((it, i) => ({
			'@type': 'ListItem',
			position: i + 1,
			name: it.name,
			item: absoluteUrl(it.url)
		}))
	};
}

export function productLd(p: {
	name: string;
	description: string;
	priceCents: number;
	slug: string;
	rating?: { value: number; count: number };
}): JsonLd {
	return {
		'@context': 'https://schema.org',
		'@type': 'Product',
		name: p.name,
		description: p.description,
		brand: { '@type': 'Brand', name: SITE_NAME },
		offers: {
			'@type': 'Offer',
			url: absoluteUrl(`/indicators/${p.slug}`),
			priceCurrency: 'USD',
			price: (p.priceCents / 100).toFixed(2),
			availability: 'https://schema.org/InStock'
		},
		...(p.rating
			? {
					aggregateRating: {
						'@type': 'AggregateRating',
						ratingValue: p.rating.value,
						reviewCount: p.rating.count
					}
				}
			: {})
	};
}

export function courseLd(c: {
	name: string;
	description: string;
	slug: string;
	priceCents: number;
}): JsonLd {
	return {
		'@context': 'https://schema.org',
		'@type': 'Course',
		name: c.name,
		description: c.description,
		provider: {
			'@type': 'Organization',
			name: SITE_NAME,
			sameAs: SITE_URL
		},
		offers: {
			'@type': 'Offer',
			category: 'Paid',
			priceCurrency: 'USD',
			price: (c.priceCents / 100).toFixed(2)
		},
		hasCourseInstance: {
			'@type': 'CourseInstance',
			courseMode: 'Online',
			courseWorkload: 'PT12H'
		},
		url: absoluteUrl(`/courses/${c.slug}`)
	};
}

export function serviceLd(
	plans: Array<{ name: string; priceCents: number; cadence: string }>
): JsonLd {
	return {
		'@context': 'https://schema.org',
		'@type': 'Service',
		serviceType: 'Trading Subscription',
		provider: { '@id': `${SITE_URL}/#organization` },
		areaServed: 'Worldwide',
		offers: plans.map((pl) => ({
			'@type': 'Offer',
			name: pl.name,
			priceCurrency: 'USD',
			price: (pl.priceCents / 100).toFixed(2),
			priceSpecification: {
				'@type': 'UnitPriceSpecification',
				billingIncrement: pl.cadence
			}
		}))
	};
}

export function faqLd(items: Array<{ q: string; a: string }>): JsonLd {
	return {
		'@context': 'https://schema.org',
		'@type': 'FAQPage',
		mainEntity: items.map((it) => ({
			'@type': 'Question',
			name: it.q,
			acceptedAnswer: { '@type': 'Answer', text: it.a }
		}))
	};
}

export function articleLd(a: {
	headline: string;
	description: string;
	url: string;
	image?: string;
	datePublished?: string;
	dateModified?: string;
	author?: string;
}): JsonLd {
	return {
		'@context': 'https://schema.org',
		'@type': 'Article',
		headline: a.headline,
		description: a.description,
		image: a.image ? absoluteUrl(a.image) : undefined,
		datePublished: a.datePublished,
		dateModified: a.dateModified ?? a.datePublished,
		author: { '@type': 'Person', name: a.author ?? SITE_NAME },
		publisher: { '@id': `${SITE_URL}/#organization` },
		mainEntityOfPage: { '@type': 'WebPage', '@id': absoluteUrl(a.url) }
	};
}

export function personLd(p: {
	name: string;
	jobTitle: string;
	url?: string;
	image?: string;
	sameAs?: string[];
}): JsonLd {
	return {
		'@context': 'https://schema.org',
		'@type': 'Person',
		name: p.name,
		jobTitle: p.jobTitle,
		url: p.url ? absoluteUrl(p.url) : undefined,
		image: p.image ? absoluteUrl(p.image) : undefined,
		worksFor: { '@id': `${SITE_URL}/#organization` },
		sameAs: p.sameAs
	};
}
