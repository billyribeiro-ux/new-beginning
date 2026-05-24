<script lang="ts">
	import {
		IconBrandX,
		IconBrandYoutube,
		IconBrandLinkedin,
		IconBrandDiscord,
		IconArrowRight
	} from '@tabler/icons-svelte';
	import LogoWordmark from '$lib/components/media/LogoWordmark.svelte';
	import { FOOTER_NAV } from '$lib/data/navigation.js';
	import { resolve } from '$app/paths';

	const year = new Date().getFullYear();
</script>

<footer class="footer">
	<div class="footer-top">
		<div class="brand-col">
			<LogoWordmark size={40} tagline />
			<p class="brand-copy">
				Lorem ipsum dolor sit amet, consectetur adipiscing elit. Engineered tooling and education
				for traders who demand process, precision, and patience.
			</p>
			<div class="socials">
				<a href="https://x.com" aria-label="X (Twitter)" rel="noopener noreferrer" target="_blank"
					><IconBrandX size={18} /></a
				>
				<a href="https://youtube.com" aria-label="YouTube" rel="noopener noreferrer" target="_blank"
					><IconBrandYoutube size={18} /></a
				>
				<a
					href="https://linkedin.com"
					aria-label="LinkedIn"
					rel="noopener noreferrer"
					target="_blank"><IconBrandLinkedin size={18} /></a
				>
				<a href="https://discord.gg" aria-label="Discord" rel="noopener noreferrer" target="_blank"
					><IconBrandDiscord size={18} /></a
				>
			</div>
		</div>

		<div class="links">
			{#each FOOTER_NAV as section (section.heading)}
				<div class="link-col">
					<h4>{section.heading}</h4>
					<ul>
						{#each section.items as item (item.href)}
							<li><a href={resolve(item.href as unknown as '/')}>{item.label}</a></li>
						{/each}
					</ul>
				</div>
			{/each}
		</div>

		<div class="cta-col">
			<p class="eyebrow">Free download</p>
			<h4>Options Greeks Guide</h4>
			<p>The 24-page PDF we wish we had on day one — delta, gamma, theta, vega, demystified.</p>
			<a href={resolve('/free-guide')} class="cta">
				Get the guide
				<IconArrowRight size={16} />
			</a>
		</div>
	</div>

	<div class="footer-bottom">
		<p>© {year} TradeFlex Trading LLC. All rights reserved.</p>
		<p class="risk">
			<strong>Risk disclosure:</strong> Trading involves substantial risk and is not suitable for every
			investor. Past performance is no guarantee of future results. This site is for educational purposes
			only.
		</p>
	</div>
</footer>

<style>
	.footer {
		background: linear-gradient(180deg, var(--surface-0), #050507);
		border-top: 1px solid var(--border-subtle);
		color: var(--ink-300);
		padding-top: clamp(3rem, 6vw, 5rem);
	}
	.footer-top {
		max-width: var(--container-max);
		margin-inline: auto;
		padding-inline: var(--container-gutter);
		display: grid;
		grid-template-columns: 1fr;
		gap: clamp(2rem, 4vw, 3.5rem);
		padding-bottom: clamp(2rem, 4vw, 3rem);
	}
	.brand-col {
		max-width: 360px;
	}
	.brand-copy {
		margin-top: var(--space-5);
		font-size: var(--text-sm);
		color: var(--ink-300);
		line-height: var(--leading-relaxed);
	}
	.socials {
		margin-top: var(--space-6);
		display: flex;
		gap: var(--space-2);
	}
	.socials a {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 40px;
		height: 40px;
		background: var(--surface-2);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-full);
		color: var(--ink-200);
		transition: all var(--dur-fast) var(--ease-out);
	}
	.socials a:hover {
		color: var(--gold-300);
		border-color: var(--border-gold);
		transform: translateY(-2px);
	}

	.links {
		display: grid;
		grid-template-columns: repeat(2, 1fr);
		gap: var(--space-8) var(--space-6);
	}

	.link-col h4 {
		font-family: var(--font-body);
		font-size: var(--text-xs);
		font-weight: var(--weight-semibold);
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		color: var(--ink-400);
		margin-bottom: var(--space-4);
	}
	.link-col ul {
		list-style: none;
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
	}
	.link-col a {
		color: var(--ink-200);
		text-decoration: none;
		font-size: var(--text-sm);
		transition: color var(--dur-fast) var(--ease-out);
	}
	.link-col a:hover {
		color: var(--gold-300);
	}

	.cta-col {
		padding: var(--space-6);
		background: var(--surface-1);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-xl);
	}
	.cta-col .eyebrow {
		font-size: var(--text-2xs);
		font-weight: var(--weight-semibold);
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		color: var(--gold-400);
		margin-bottom: var(--space-2);
	}
	.cta-col .eyebrow::before {
		display: none;
	}
	.cta-col h4 {
		font-family: var(--font-display);
		font-size: var(--text-xl);
		margin: 0 0 var(--space-3);
		color: var(--ink-100);
	}
	.cta-col p {
		font-size: var(--text-sm);
		color: var(--ink-300);
		line-height: var(--leading-relaxed);
		margin: 0 0 var(--space-5);
	}
	.cta {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
		padding: var(--space-3) var(--space-5);
		background: var(--gradient-gold);
		color: var(--surface-0);
		font-weight: var(--weight-semibold);
		font-size: var(--text-sm);
		border-radius: var(--radius-full);
		text-decoration: none;
		transition:
			transform var(--dur-base) var(--ease-out),
			gap var(--dur-base) var(--ease-out);
	}
	.cta:hover {
		transform: translateY(-1px);
		gap: var(--space-3);
	}

	.footer-bottom {
		border-top: 1px solid var(--border-subtle);
		padding: var(--space-6) var(--container-gutter);
		max-width: var(--container-max);
		margin-inline: auto;
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
		font-size: var(--text-xs);
	}
	.footer-bottom p {
		margin: 0;
		color: var(--ink-400);
	}
	.risk {
		line-height: var(--leading-relaxed);
		max-width: 720px;
	}
	.risk strong {
		color: var(--ink-200);
	}

	@media (min-width: 768px) {
		.footer-top {
			grid-template-columns: 1.2fr 2fr;
		}
		.links {
			grid-template-columns: repeat(3, 1fr);
		}
	}
	@media (min-width: 1280px) {
		.footer-top {
			grid-template-columns: 1.1fr 2fr 1.1fr;
		}
	}
</style>
