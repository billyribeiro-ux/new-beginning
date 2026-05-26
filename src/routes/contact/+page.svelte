<script lang="ts">
	import IconMail from '@tabler/icons-svelte/icons/mail';
	import IconMapPin from '@tabler/icons-svelte/icons/map-pin';
	import IconBrandDiscord from '@tabler/icons-svelte/icons/brand-discord';
	import IconClock from '@tabler/icons-svelte/icons/clock';
	import Seo from '$lib/components/seo/Seo.svelte';
	import Breadcrumbs from '$lib/components/seo/Breadcrumbs.svelte';
	import SectionHeading from '$lib/components/marketing/SectionHeading.svelte';
	import FAQSection from '$lib/components/marketing/FAQSection.svelte';
	import ContactForm from '$lib/components/forms/ContactForm.svelte';
	import { GENERAL_FAQS } from '$lib/data/faqs.js';
	import { fadeUp } from '$lib/animations/attachments.js';
</script>

<Seo
	title="Contact"
	description="Reach the TradeFlex Trading desk. Support replies within one business day."
	type="website"
/>

<section class="contact-hero">
	<div class="container">
		<Breadcrumbs
			items={[
				{ label: 'Home', href: '/' },
				{ label: 'Contact', href: '/contact' }
			]}
		/>
		<div class="hero" {@attach fadeUp({ y: 20 })}>
			<p class="eyebrow">Contact</p>
			<h1>Tell us what you are working on.</h1>
			<p class="lead">
				Lorem ipsum dolor sit amet, consectetur adipiscing elit. Whether you have a question about
				Revolution Ranger, Options 101, or you are exploring a team license — drop us a line. We
				reply within one business day.
			</p>
		</div>
	</div>
</section>

<section class="section">
	<div class="container">
		<div class="contact-grid">
			<aside class="info" {@attach fadeUp({ y: 18 })}>
				<h2>How to reach us</h2>
				<ul class="info-list">
					<li>
						<span class="ic"><IconMail size={18} /></span>
						<div>
							<p class="lbl">Email</p>
							<a href="mailto:hello@tradeflextrading.com">hello@tradeflextrading.com</a>
						</div>
					</li>
					<li>
						<span class="ic"><IconBrandDiscord size={18} /></span>
						<div>
							<p class="lbl">Community</p>
							<a href="https://discord.gg" rel="noopener noreferrer" target="_blank"
								>Join the public Discord</a
							>
						</div>
					</li>
					<li>
						<span class="ic"><IconClock size={18} /></span>
						<div>
							<p class="lbl">Hours</p>
							<p>Mon–Fri · 8am–6pm ET</p>
						</div>
					</li>
					<li>
						<span class="ic"><IconMapPin size={18} /></span>
						<div>
							<p class="lbl">Mailing</p>
							<p>1234 Lakeside Ave, Suite 600<br />Chicago, IL 60601 · USA</p>
						</div>
					</li>
				</ul>
			</aside>

			<div class="form-card" {@attach fadeUp({ y: 24, delay: 0.1 })}>
				<h2>Send a message</h2>
				<p class="muted">Most inquiries are answered within one business day.</p>
				<ContactForm />
			</div>
		</div>
	</div>
</section>

<section class="section">
	<div class="container">
		<SectionHeading eyebrow="Before you ask…" title="A few answers up front." align="center" />
		<div style="margin-top: clamp(2rem, 5vw, 3rem);">
			<FAQSection items={GENERAL_FAQS} />
		</div>
	</div>
</section>

<style>
	.contact-hero {
		padding: clamp(4rem, 8vw, 6rem) 0;
		background: radial-gradient(ellipse at center top, rgba(232, 182, 96, 0.1), transparent 60%);
	}
	.hero {
		max-width: 720px;
		margin-top: var(--space-6);
	}
	.eyebrow {
		font-size: var(--text-2xs);
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		color: var(--gold-400);
		font-weight: var(--weight-semibold);
		margin: 0;
	}
	.eyebrow::before {
		display: none;
	}
	h1 {
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
		padding-block: clamp(4rem, 8vw, 7rem);
	}

	.contact-grid {
		display: grid;
		grid-template-columns: 1fr;
		gap: clamp(2rem, 4vw, 3rem);
	}
	@media (--bp-lg) {
		.contact-grid {
			grid-template-columns: 1fr 1.4fr;
		}
	}

	.info h2 {
		font-family: var(--font-display);
		font-size: var(--text-2xl);
		margin: 0 0 var(--space-6);
	}
	.info-list {
		list-style: none;
		display: flex;
		flex-direction: column;
		gap: var(--space-5);
	}
	.info-list li {
		display: grid;
		grid-template-columns: 40px 1fr;
		gap: var(--space-3);
	}
	.info-list .ic {
		width: 40px;
		height: 40px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		background: var(--surface-2);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-full);
		color: var(--gold-400);
	}
	.info-list .lbl {
		font-size: var(--text-2xs);
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		color: var(--ink-400);
		font-weight: var(--weight-semibold);
		margin: 0;
	}
	.info-list a {
		color: var(--ink-100);
		font-weight: var(--weight-medium);
	}
	.info-list a:hover {
		color: var(--gold-300);
		text-decoration: underline;
	}
	.info-list p {
		font-size: var(--text-sm);
		color: var(--ink-200);
		margin: 4px 0 0;
		line-height: var(--leading-relaxed);
	}

	.form-card {
		padding: clamp(2rem, 4vw, 3rem);
		background: var(--surface-1);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-2xl);
	}
	.form-card h2 {
		font-family: var(--font-display);
		font-size: var(--text-2xl);
		margin: 0 0 var(--space-2);
	}
	.muted {
		color: var(--ink-400);
		font-size: var(--text-sm);
		margin: 0 0 var(--space-6);
	}
</style>
