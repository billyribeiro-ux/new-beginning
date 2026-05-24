<script lang="ts">
	import {
		IconCompass,
		IconTargetArrow,
		IconBolt,
		IconShieldCheck,
		IconBook2,
		IconUsersGroup
	} from '@tabler/icons-svelte';
	import Seo from '$lib/components/seo/Seo.svelte';
	import JsonLd from '$lib/components/seo/JsonLd.svelte';
	import Breadcrumbs from '$lib/components/seo/Breadcrumbs.svelte';
	import SectionHeading from '$lib/components/marketing/SectionHeading.svelte';
	import StatBlock from '$lib/components/marketing/StatBlock.svelte';
	import CTABanner from '$lib/components/marketing/CTABanner.svelte';
	import Badge from '$lib/components/ui/Badge.svelte';
	import { personLd } from '$lib/utils/jsonld.js';
	import { fadeUp, stagger } from '$lib/animations/attachments.js';

	const principles = [
		{
			icon: IconCompass,
			title: 'Process beats prediction',
			body: 'Lorem ipsum dolor sit amet. We build for repeatability, not for the highest-conviction call of the week.'
		},
		{
			icon: IconTargetArrow,
			title: 'Evidence over ego',
			body: 'Sed do eiusmod tempor. Every system, every change, every alert is backtested or it does not ship.'
		},
		{
			icon: IconBolt,
			title: 'Latency matters',
			body: 'Ut enim ad minim. Sub-millisecond signals. Lazy loading is for content sites, not for tape.'
		},
		{
			icon: IconShieldCheck,
			title: 'Risk is the product',
			body: 'Duis aute irure dolor. We teach traders to size and to stop. Edge without survival is theatre.'
		},
		{
			icon: IconBook2,
			title: 'Teach the work',
			body: 'Excepteur sint occaecat. We open the playbook because secrets do not compound — process does.'
		},
		{
			icon: IconUsersGroup,
			title: 'Cohort, not crowd',
			body: 'Sunt in culpa qui officia. A focused desk of operators is worth more than a thousand-member chatroom.'
		}
	];

	const team = [
		{
			name: 'Jordan Vega',
			role: 'Founder · Head of Desk',
			initials: 'JV',
			bio: 'Lorem ipsum dolor sit amet. 14 years on prop and family-office desks. Specializes in futures and convex options structures.'
		},
		{
			name: 'Casey Lin',
			role: 'Lead Quant',
			initials: 'CL',
			bio: 'Sed do eiusmod tempor. Builds and validates every indicator that ships. Former systematic researcher at a Tier-1 fund.'
		},
		{
			name: 'Priya Shah',
			role: 'Director of Education',
			initials: 'PS',
			bio: 'Ut enim ad minim veniam. Designs the cohort experience end-to-end. Twelve years teaching options to professional traders.'
		}
	];

	const milestones = [
		{
			year: '2026',
			title: 'TradeFlex Trading founded',
			body: 'Lorem ipsum dolor sit amet. Born from a single conviction: traders deserve tooling and teachers that respect their time.'
		},
		{
			year: '2026',
			title: 'Revolution Ranger ships v1',
			body: 'Sed do eiusmod tempor incididunt ut labore et dolore magna. The compression-to-expansion engine goes live for NinjaTrader and TradingView.'
		},
		{
			year: '2026',
			title: 'Options 101 opens first cohort',
			body: 'Ut enim ad minim veniam, quis nostrud exercitation. Twelve modules, exam-graded, with weekly office hours.'
		},
		{
			year: 'In progress',
			title: 'Live desk + new indicators',
			body: 'Duis aute irure dolor in reprehenderit. We are shipping Liquidity Hawk and the Day Trading desk continues to scale.'
		}
	];

	const teamStats = [
		{ value: 14, label: 'Years on desk' },
		{ value: 14200, label: 'Active members' },
		{
			value: 1.4,
			label: 'Million signals delivered',
			formatter: (n: number) => n.toFixed(1) + 'M'
		},
		{ value: 99.97, label: 'Platform uptime', formatter: (n: number) => n.toFixed(2) + '%' }
	];
</script>

<Seo
	title="About"
	description="The team, the principles, and the timeline behind TradeFlex Trading."
	type="article"
/>

<JsonLd
	data={personLd({ name: 'Jordan Vega', jobTitle: 'Founder & Head of Desk', url: '/about' })}
/>

<section class="about-hero">
	<div class="container">
		<Breadcrumbs
			items={[
				{ label: 'Home', href: '/' },
				{ label: 'About', href: '/about' }
			]}
		/>
		<div class="hero-inner" {@attach fadeUp({ y: 20 })}>
			<Badge variant="gold">About TradeFlex</Badge>
			<h1>We build the tools we use ourselves.</h1>
			<p class="lead">
				Lorem ipsum dolor sit amet, consectetur adipiscing elit. TradeFlex Trading exists for a
				single kind of trader: the one who wants a process they can defend, indicators they can
				trust, and an operator-grade desk to sit at every morning.
			</p>
		</div>
	</div>
</section>

<section class="section">
	<div class="container">
		<SectionHeading
			eyebrow="What we believe"
			title="Six principles we will not compromise."
			align="center"
		/>
		<div class="principles" {@attach stagger({ stagger: 0.06, y: 18 })}>
			{#each principles as p (p.title)}
				<article class="principle">
					<span class="ic"><p.icon size={22} /></span>
					<h3>{p.title}</h3>
					<p>{p.body}</p>
				</article>
			{/each}
		</div>
	</div>
</section>

<section class="section bg-dark">
	<div class="container">
		<SectionHeading eyebrow="The desk" title="People behind the playbook." align="center" />
		<div class="team-grid" {@attach stagger({ stagger: 0.1, y: 20 })}>
			{#each team as t (t.name)}
				<article class="team-card">
					<div class="avatar">{t.initials}</div>
					<h3>{t.name}</h3>
					<p class="role">{t.role}</p>
					<p class="bio">{t.bio}</p>
				</article>
			{/each}
		</div>

		<div style="margin-top: clamp(3rem, 6vw, 5rem)">
			<StatBlock stats={teamStats} />
		</div>
	</div>
</section>

<section class="section">
	<div class="container">
		<SectionHeading eyebrow="Timeline" title="How we got here." align="center" />
		<ol class="timeline" {@attach stagger({ stagger: 0.08, y: 18 })}>
			{#each milestones as m (m.title)}
				<li>
					<span class="year">{m.year}</span>
					<div class="dot"></div>
					<div class="body">
						<h3>{m.title}</h3>
						<p>{m.body}</p>
					</div>
				</li>
			{/each}
		</ol>
	</div>
</section>

<section class="section">
	<div class="container">
		<CTABanner
			eyebrow="Join the desk"
			title="If you build a process, the trades take care of themselves."
			subtitle="Two-week refund. Cancel anytime. The room opens Monday at 9:30am ET."
			ctaLabel="See the subscription"
			ctaHref="/subscription"
			secondaryLabel="Talk to us"
			secondaryHref="/contact"
		/>
	</div>
</section>

<style>
	.about-hero {
		padding: clamp(4rem, 8vw, 6rem) 0;
		background: radial-gradient(ellipse at center top, rgba(232, 182, 96, 0.1), transparent 60%);
	}
	.hero-inner {
		max-width: 760px;
		margin-top: var(--space-6);
	}
	.hero-inner h1 {
		font-family: var(--font-display);
		font-size: clamp(2.5rem, 5vw, 4.5rem);
		margin: var(--space-5) 0 var(--space-4);
		letter-spacing: var(--tracking-tight);
		line-height: var(--leading-tight);
	}
	.lead {
		font-size: var(--text-md);
		color: var(--ink-300);
		line-height: var(--leading-relaxed);
		margin: 0;
	}

	.section {
		padding-block: clamp(4rem, 8vw, 8rem);
	}
	.bg-dark {
		background: linear-gradient(180deg, var(--surface-0), #050507);
		border-block: 1px solid var(--border-subtle);
	}

	.principles {
		margin-top: clamp(2.5rem, 5vw, 4rem);
		display: grid;
		grid-template-columns: 1fr;
		gap: var(--space-4);
	}
	@media (min-width: 640px) {
		.principles {
			grid-template-columns: repeat(2, 1fr);
		}
	}
	@media (min-width: 1024px) {
		.principles {
			grid-template-columns: repeat(3, 1fr);
		}
	}
	.principle {
		padding: var(--space-7);
		background: var(--surface-1);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-xl);
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
	}
	.principle .ic {
		width: 48px;
		height: 48px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		background: linear-gradient(135deg, rgba(232, 182, 96, 0.16), rgba(212, 162, 76, 0.04));
		color: var(--gold-300);
		border: 1px solid var(--border-gold);
		border-radius: var(--radius-md);
		margin-bottom: var(--space-3);
	}
	.principle h3 {
		font-family: var(--font-display);
		font-size: var(--text-xl);
		margin: 0;
	}
	.principle p {
		font-size: var(--text-sm);
		color: var(--ink-300);
		line-height: var(--leading-relaxed);
		margin: 0;
	}

	.team-grid {
		margin-top: clamp(2.5rem, 5vw, 4rem);
		display: grid;
		grid-template-columns: 1fr;
		gap: var(--space-5);
	}
	@media (min-width: 768px) {
		.team-grid {
			grid-template-columns: repeat(3, 1fr);
		}
	}
	.team-card {
		text-align: center;
		padding: var(--space-7);
		background: var(--surface-1);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-xl);
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-3);
	}
	.team-card .avatar {
		width: 80px;
		height: 80px;
		border-radius: var(--radius-full);
		background: var(--gradient-gold);
		color: var(--surface-0);
		display: inline-flex;
		align-items: center;
		justify-content: center;
		font-family: var(--font-display);
		font-weight: var(--weight-bold);
		font-size: var(--text-xl);
		margin-bottom: var(--space-3);
	}
	.team-card h3 {
		font-family: var(--font-display);
		font-size: var(--text-xl);
		margin: 0;
	}
	.team-card .role {
		font-size: var(--text-xs);
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		color: var(--gold-400);
		font-weight: var(--weight-semibold);
		margin: 0;
	}
	.team-card .bio {
		font-size: var(--text-sm);
		color: var(--ink-300);
		line-height: var(--leading-relaxed);
		margin: 0;
	}

	.timeline {
		list-style: none;
		max-width: 720px;
		margin: clamp(2.5rem, 5vw, 4rem) auto 0;
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
		position: relative;
	}
	.timeline::before {
		content: '';
		position: absolute;
		left: 64px;
		top: 8px;
		bottom: 8px;
		width: 2px;
		background: linear-gradient(180deg, var(--gold-500), transparent);
		opacity: 0.3;
	}
	.timeline li {
		display: grid;
		grid-template-columns: 56px 16px 1fr;
		gap: var(--space-3);
		align-items: start;
	}
	.year {
		font-family: var(--font-mono);
		font-size: var(--text-xs);
		color: var(--gold-400);
		font-weight: var(--weight-semibold);
		text-transform: uppercase;
		letter-spacing: var(--tracking-wider);
		padding-top: 8px;
	}
	.dot {
		width: 16px;
		height: 16px;
		background: var(--gold-500);
		border-radius: 50%;
		margin-top: 6px;
		box-shadow:
			0 0 0 4px var(--surface-0),
			0 0 0 5px var(--border-gold);
		position: relative;
		z-index: 1;
	}
	.body {
		padding: var(--space-5);
		background: var(--surface-1);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-lg);
	}
	.body h3 {
		font-family: var(--font-display);
		font-size: var(--text-md);
		margin: 0 0 var(--space-2);
	}
	.body p {
		font-size: var(--text-sm);
		color: var(--ink-300);
		line-height: var(--leading-relaxed);
		margin: 0;
	}
</style>
