export type ProductKind = 'indicator' | 'course';

export interface Product {
	id: string;
	slug: string;
	kind: ProductKind;
	name: string;
	tagline: string;
	description: string;
	priceCents: number;
	originalPriceCents?: number;
	badge?: string;
	rating: { value: number; count: number };
	highlights: string[];
	features: Array<{ title: string; description: string; icon: string }>;
	specs?: Array<{ label: string; value: string }>;
	deliverables?: string[];
	requirements?: string[];
	media: { posterColor: string; accent: string };
}

export const INDICATORS: Product[] = [
	{
		id: 'prod_revolution_ranger',
		slug: 'revolution-ranger',
		kind: 'indicator',
		name: 'Revolution Ranger',
		tagline: 'Adaptive multi-timeframe range detection — engineered for serious day traders.',
		description:
			'Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Revolution Ranger combines volatility-weighted range mapping with intraday liquidity scoring to surface high-probability inflection zones before they crystallize on the tape. Built for NinjaTrader, TradingView, and ThinkOrSwim — install in under three minutes.',
		priceCents: 99700,
		originalPriceCents: 129700,
		badge: 'Best Seller',
		rating: { value: 4.9, count: 1284 },
		highlights: [
			'Detects compression-to-expansion regime shifts in real time',
			'Auto-calibrates to ES, NQ, CL, GC, and 40+ instruments',
			'Layered confluence with volume profile + market structure',
			'Lifetime updates, free indicator family access'
		],
		features: [
			{
				icon: 'IconChartCandle',
				title: 'Adaptive Range Engine',
				description:
					'Lorem ipsum dolor sit amet, consectetur adipiscing elit. The proprietary engine adapts band width to realized volatility every tick — no manual tuning, ever.'
			},
			{
				icon: 'IconBolt',
				title: 'Sub-millisecond Signals',
				description:
					'Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Optimized DOM hooks deliver alerts faster than the next candle prints.'
			},
			{
				icon: 'IconShieldCheck',
				title: 'Trade-Validated Logic',
				description:
					'Ut enim ad minim veniam, quis nostrud exercitation. Built and refined across six market regimes by a desk that trades it live.'
			},
			{
				icon: 'IconStack2',
				title: 'Confluence Layers',
				description:
					'Duis aute irure dolor in reprehenderit. Compose Revolution Ranger with liquidity sweeps, order flow, and your favorite tools.'
			},
			{
				icon: 'IconAdjustments',
				title: 'Full Customization',
				description:
					'Excepteur sint occaecat cupidatat non proident. Every threshold, color, alert, and zone is yours to shape.'
			},
			{
				icon: 'IconRefresh',
				title: 'Lifetime Updates',
				description:
					'Sunt in culpa qui officia deserunt. New regime detectors, instrument presets, and platform exports — forever.'
			}
		],
		specs: [
			{ label: 'Platforms', value: 'NinjaTrader 8, TradingView, ThinkOrSwim' },
			{ label: 'License', value: '1 personal computer · transferable' },
			{ label: 'Updates', value: 'Lifetime' },
			{ label: 'Support', value: 'Priority email + Discord' },
			{ label: 'Refund', value: '14-day money-back' }
		],
		deliverables: [
			'Encrypted indicator package (NT8 .zip, TradingView Pine script v6, TOS study)',
			'30-page strategy playbook (PDF)',
			'Onboarding session (45 min, 1:1)',
			'Private Discord channel'
		],
		requirements: [
			'NinjaTrader 8.1+ / TradingView Pro+ / ThinkOrSwim',
			'A funded or sim account',
			'Curiosity, patience, and a willingness to backtest'
		],
		media: { posterColor: 'linear-gradient(135deg, #2a1f0f, #0a0a0b)', accent: '#E8B660' }
	}
];

export const COURSES: Product[] = [
	{
		id: 'prod_options_101',
		slug: 'options-101',
		kind: 'course',
		name: 'Options 101',
		tagline: 'From greeks to verticals — a structured foundation for the modern options trader.',
		description:
			'Lorem ipsum dolor sit amet, consectetur adipiscing elit. Options 101 is a twelve-module masterclass that walks you from contract mechanics through delta-neutral hedging using interactive payoff visualizations, live-market case studies, and a 40-question certification exam. Designed for traders who want to stop guessing and start engineering positions.',
		priceCents: 99700,
		badge: 'Cohort Open',
		rating: { value: 4.8, count: 612 },
		highlights: [
			'12 modules · 48 lessons · 12 hours of HD video',
			'Interactive payoff & greeks playgrounds',
			'8 live case studies on ES, SPY, AAPL, NVDA, TLT',
			'Certificate of completion + alumni Discord'
		],
		features: [
			{
				icon: 'IconBook2',
				title: 'Foundations First',
				description:
					'Lorem ipsum dolor sit amet. Mechanics, settlement, assignment, multipliers — built bottom-up so nothing is hand-waved.'
			},
			{
				icon: 'IconMath',
				title: 'Greeks Without the Calculus',
				description:
					'Consectetur adipiscing elit. Visual-first treatment of delta, gamma, theta, vega, rho — with playable scenarios.'
			},
			{
				icon: 'IconChartLine',
				title: 'Strategy Construction',
				description:
					'Sed do eiusmod. From long calls to broken-wing butterflies — when, why, and how each structure earns its keep.'
			},
			{
				icon: 'IconCalculator',
				title: 'Position Sizing',
				description:
					'Ut labore et dolore. A repeatable framework for sizing with respect to portfolio vol and assignment risk.'
			},
			{
				icon: 'IconCertificate',
				title: 'Certification Exam',
				description:
					'Magna aliqua. Pass the 40-question final to earn a signed certificate and access alumni-only content.'
			},
			{
				icon: 'IconUsersGroup',
				title: 'Cohort Community',
				description:
					'Quis nostrud exercitation. Weekly office hours plus a focused Discord — built for traders who push each other.'
			}
		],
		specs: [
			{ label: 'Format', value: '12 modules · self-paced · cohort kickoffs monthly' },
			{ label: 'Length', value: '~12 hours video + exercises' },
			{ label: 'Level', value: 'Beginner → Intermediate' },
			{ label: 'Access', value: 'Lifetime, all updates' },
			{ label: 'Certificate', value: 'Yes (after passing exam)' }
		],
		deliverables: [
			'Lifetime access to all video lessons and updates',
			'Interactive playgrounds (greeks, payoff diagrams, scenarios)',
			'40-question certification exam',
			'Alumni Discord + monthly live Q&A'
		],
		requirements: [
			'A brokerage account that supports options (paper account is fine)',
			'Basic familiarity with stock-market mechanics',
			'4–6 hours per week for two to three weeks'
		],
		media: { posterColor: 'linear-gradient(135deg, #1a2233, #0a0a0b)', accent: '#F5D08A' }
	}
];

export function getAllProducts(): Product[] {
	return [...INDICATORS, ...COURSES];
}

export function getProductBySlug(slug: string): Product | undefined {
	return getAllProducts().find((p) => p.slug === slug);
}

export function getProductsByKind(kind: ProductKind): Product[] {
	return getAllProducts().filter((p) => p.kind === kind);
}
