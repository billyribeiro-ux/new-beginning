export interface FAQ {
	q: string;
	a: string;
}

export const GENERAL_FAQS: FAQ[] = [
	{
		q: 'Who is TradeFlex Trading for?',
		a: 'Lorem ipsum dolor sit amet, consectetur adipiscing elit. We build for active day traders, options traders, and committed self-directed investors who want a disciplined, data-driven process. If you trade live capital or are working toward a funded account, this is built for you.'
	},
	{
		q: 'Do you give trade signals?',
		a: 'Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. We provide setups, levels, and real-time commentary in the live room, but never blind signals. Our philosophy is to make you a better operator, not a button-pusher.'
	},
	{
		q: 'What platforms do your indicators support?',
		a: 'Ut enim ad minim veniam, quis nostrud exercitation. Revolution Ranger ships for NinjaTrader 8, TradingView (Pine v6), and ThinkOrSwim. Additional platforms are added quarterly based on member demand.'
	},
	{
		q: 'Is there a refund policy?',
		a: 'Duis aute irure dolor. Indicator and course purchases include a 14-day money-back guarantee. The Day Trading subscription is cancelable anytime with no penalty — you keep access through the end of the billing period.'
	},
	{
		q: 'Do I need experience to start?',
		a: 'Excepteur sint occaecat. The Options 101 course is beginner-friendly. Revolution Ranger and the Day Trading subscription assume basic familiarity with order entry, charting, and risk management.'
	},
	{
		q: 'Is this financial advice?',
		a: 'No. Sunt in culpa qui officia. TradeFlex Trading provides educational content and analytical tools only. Nothing on this site is personalized investment advice — please consult a licensed advisor for your situation.'
	}
];

export const SUBSCRIPTION_FAQS: FAQ[] = [
	{
		q: 'What is included in the Day Trading subscription?',
		a: 'Lorem ipsum dolor sit amet. Daily pre-market briefings, the live trading room from 9:30am to 12:00pm ET, real-time alerts on ES/NQ/CL/GC, the weekly trade review, and the member-only Discord. Higher tiers add 1:1 reviews and bundled products.'
	},
	{
		q: 'Can I switch plans later?',
		a: 'Yes. Sed do eiusmod tempor. You can upgrade at any time and the difference is prorated. Downgrades take effect at the next renewal.'
	},
	{
		q: 'How do I cancel?',
		a: 'Cancel anytime from your Dashboard → Subscription. Ut labore et dolore. You keep access through the end of the billing period. No retention calls, no friction.'
	},
	{
		q: 'Do you offer team or firm pricing?',
		a: 'Yes — magna aliqua. Reach out via the Contact page and we will tailor a license for your desk.'
	}
];

export const COURSE_FAQS: FAQ[] = [
	{
		q: 'How long does Options 101 take to complete?',
		a: 'Most students finish in two to three weeks at four to six hours per week. Lorem ipsum dolor sit amet. The course is self-paced and your access is lifetime.'
	},
	{
		q: 'Is there a certificate?',
		a: 'Yes. After passing the 40-question final exam you receive a signed certificate of completion and unlock the alumni Discord.'
	},
	{
		q: 'Will this work for non-US markets?',
		a: 'The mechanics chapters are universal. Consectetur adipiscing elit. The case studies use US instruments (SPY, AAPL, NVDA, TLT) but the framework transfers cleanly to European and APAC names.'
	}
];

export const INDICATOR_FAQS: FAQ[] = [
	{
		q: 'How fast is installation?',
		a: 'Under three minutes on any supported platform. We provide a one-click installer for NinjaTrader and copy-paste Pine for TradingView. Sed do eiusmod tempor.'
	},
	{
		q: 'Does Revolution Ranger repaint?',
		a: 'No. Signals are confirmed at bar close and locked. Historical signals match live signals exactly.'
	},
	{
		q: 'Can I run it on multiple machines?',
		a: 'Your license covers one personal computer at a time and is freely transferable. Ut enim ad minim veniam.'
	}
];
