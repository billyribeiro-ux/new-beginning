export interface Testimonial {
	id: string;
	name: string;
	role: string;
	initials: string;
	avatarColor: string;
	rating: number;
	quote: string;
	context?: string;
}

export const TESTIMONIALS: Testimonial[] = [
	{
		id: 't_01',
		name: 'Marcus A.',
		role: 'Prop Trader · 8 yrs',
		initials: 'MA',
		avatarColor: 'linear-gradient(135deg, #E8B660, #B0832F)',
		rating: 5,
		quote:
			'Lorem ipsum dolor sit amet, consectetur adipiscing elit. Revolution Ranger replaced three of my legacy indicators on day one. The compression-to-expansion call has been the cleanest setup in my deck for six months running.',
		context: 'Day Trading · Annual'
	},
	{
		id: 't_02',
		name: 'Priya N.',
		role: 'Independent Options Trader',
		initials: 'PN',
		avatarColor: 'linear-gradient(135deg, #F5D08A, #D4A24C)',
		rating: 5,
		quote:
			'Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Options 101 is the course I wish I had taken three years ago. The greeks playground alone is worth the entry.',
		context: 'Options 101 alumni'
	},
	{
		id: 't_03',
		name: 'Daniel R.',
		role: 'Futures · Energy Desk',
		initials: 'DR',
		avatarColor: 'linear-gradient(135deg, #B0832F, #876320)',
		rating: 5,
		quote:
			'Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris. The Day Trading subscription has tightened my CL execution by 28% on average. The live room is what separates this from every other service I have tried.',
		context: 'Day Trading · Quarterly'
	},
	{
		id: 't_04',
		name: 'Selena K.',
		role: 'Retail → Funded · 3 yrs',
		initials: 'SK',
		avatarColor: 'linear-gradient(135deg, #FAEDCC, #E8B660)',
		rating: 5,
		quote:
			'Duis aute irure dolor in reprehenderit in voluptate velit esse cillum. I passed my funded eval the second week after pairing Revolution Ranger with the live room. No fluff. Just process.',
		context: 'Day Trading · Annual'
	},
	{
		id: 't_05',
		name: 'Owen T.',
		role: 'Volatility Arbitrage',
		initials: 'OT',
		avatarColor: 'linear-gradient(135deg, #D4A24C, #5D4416)',
		rating: 5,
		quote:
			'Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt. Their treatment of vega and gamma exposure in Options 101 is the cleanest I have seen outside academic texts.',
		context: 'Options 101 alumni'
	}
];
