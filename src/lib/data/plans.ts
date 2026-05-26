export type Cadence = 'monthly' | 'quarterly' | 'annual';

export interface SubscriptionPlan {
	id: string;
	slug: string;
	name: string;
	cadence: Cadence;
	priceCents: number;
	monthlyEquivalentCents: number;
	savingsPct?: number;
	tagline: string;
	highlights: string[];
	featured?: boolean;
	badge?: string;
}

export const DAY_TRADING_PLANS: SubscriptionPlan[] = [
	{
		id: 'plan_dt_monthly',
		slug: 'day-trading-monthly',
		name: 'Monthly',
		cadence: 'monthly',
		priceCents: 24700,
		monthlyEquivalentCents: 24700,
		tagline: 'Maximum flexibility. Cancel anytime.',
		highlights: [
			'Daily pre-market briefings (Mon–Fri)',
			'Live trading room access · 9:30am–12:00pm ET',
			'Real-time alerts on ES, NQ, CL, GC',
			'Member-only weekly review session',
			'Cancel anytime — no contracts'
		]
	},
	{
		id: 'plan_dt_quarterly',
		slug: 'day-trading-quarterly',
		name: 'Quarterly',
		cadence: 'quarterly',
		priceCents: 69700,
		monthlyEquivalentCents: 23234,
		savingsPct: 6,
		tagline: 'Most popular. Lock in a quarter at a discount.',
		highlights: [
			'Everything in Monthly, plus:',
			'Quarterly 1:1 strategy review (45 min)',
			'Member-only macro deep-dive call',
			'Early access to new indicators in beta',
			'Lifetime price-lock on this tier'
		],
		featured: true,
		badge: 'Most Popular'
	},
	{
		id: 'plan_dt_annual',
		slug: 'day-trading-annual',
		name: 'Annual',
		cadence: 'annual',
		priceCents: 199700,
		monthlyEquivalentCents: 16642,
		savingsPct: 33,
		tagline: 'Best value. Two months free, plus exclusive perks.',
		highlights: [
			'Everything in Quarterly, plus:',
			'Revolution Ranger indicator included ($997 value)',
			'Options 101 course included ($997 value)',
			'Two 1:1 strategy reviews per year',
			'Founders Circle Discord access'
		],
		badge: 'Best Value'
	}
];

export function getPlanBySlug(slug: string): SubscriptionPlan | undefined {
	return DAY_TRADING_PLANS.find((p) => p.slug === slug);
}

export function getPlanByCadence(cadence: Cadence): SubscriptionPlan | undefined {
	return DAY_TRADING_PLANS.find((p) => p.cadence === cadence);
}
