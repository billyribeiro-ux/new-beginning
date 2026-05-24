<script lang="ts">
	import {
		IconChartLine,
		IconCpu2,
		IconChartHistogram,
		IconCertificate,
		IconShieldCheck,
		IconUsersGroup,
		IconCloudDataConnection,
		IconRocket
	} from '@tabler/icons-svelte';

	const items = [
		{ icon: IconChartLine, label: 'TradingView Pro' },
		{ icon: IconCpu2, label: 'NinjaTrader 8' },
		{ icon: IconChartHistogram, label: 'ThinkOrSwim' },
		{ icon: IconCertificate, label: 'CMT Affiliated' },
		{ icon: IconShieldCheck, label: 'SOC 2 Aligned' },
		{ icon: IconUsersGroup, label: '14,200+ Members' },
		{ icon: IconCloudDataConnection, label: 'CQG Data' },
		{ icon: IconRocket, label: 'Built by Pros' }
	];
</script>

<section class="trust" aria-label="Trusted by platforms and partners">
	<div class="container">
		<p class="eyebrow center">Built for the platforms you already trust</p>
		<div class="marquee" aria-hidden="true">
			<div class="track">
				{#each [...items, ...items] as item, i (i)}
					<div class="chip">
						<span class="i"><item.icon size={20} /></span>
						<span class="t">{item.label}</span>
					</div>
				{/each}
			</div>
		</div>
	</div>
</section>

<style>
	.trust {
		padding-block: clamp(2.5rem, 4vw, 4rem);
		border-block: 1px solid var(--border-subtle);
		background: var(--surface-0);
	}
	.center {
		text-align: center;
	}
	.eyebrow {
		font-size: var(--text-2xs);
		font-weight: var(--weight-semibold);
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		color: var(--ink-400);
		margin-bottom: var(--space-6);
	}
	.eyebrow::before {
		display: none;
	}
	.marquee {
		overflow: hidden;
		mask-image: linear-gradient(90deg, transparent, black 10%, black 90%, transparent);
		-webkit-mask-image: linear-gradient(90deg, transparent, black 10%, black 90%, transparent);
	}
	.track {
		display: flex;
		gap: var(--space-10);
		width: max-content;
		animation: marquee-left 38s linear infinite;
	}
	@media (prefers-reduced-motion: reduce) {
		.track {
			animation: none;
		}
	}
	.chip {
		display: inline-flex;
		align-items: center;
		gap: var(--space-3);
		color: var(--ink-300);
		font-family: var(--font-body);
		font-weight: var(--weight-medium);
		font-size: var(--text-sm);
		white-space: nowrap;
	}
	.i {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		border-radius: var(--radius-full);
		background: var(--surface-2);
		color: var(--gold-400);
		border: 1px solid var(--border-default);
	}
</style>
