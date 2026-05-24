<script lang="ts">
	import {
		IconDownload,
		IconKey,
		IconCpu,
		IconRefresh,
		IconBookmark,
		IconCheck
	} from '@tabler/icons-svelte';
	import Seo from '$lib/components/seo/Seo.svelte';
	import Badge from '$lib/components/ui/Badge.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Accordion from '$lib/components/ui/Accordion.svelte';
	import { INDICATORS } from '$lib/data/products.js';
	import { toasts } from '$lib/stores/toast.svelte.js';

	const owned = INDICATORS.map((i) => ({
		...i,
		licenseKey: 'TF-RR-A8K2-9MNV-X7QL-J3P0',
		downloads: [
			{ platform: 'NinjaTrader 8', version: 'v2.4.1', date: 'May 12, 2026', size: '8.4 MB' },
			{
				platform: 'TradingView (Pine v6)',
				version: 'v2.4.1',
				date: 'May 12, 2026',
				size: '1.1 KB'
			},
			{ platform: 'ThinkOrSwim', version: 'v2.4.1', date: 'May 12, 2026', size: '6.2 KB' }
		]
	}));

	const installGuide = [
		{
			id: 'i1',
			title: 'NinjaTrader 8 — install in 3 minutes',
			content:
				'Lorem ipsum dolor sit amet. Download the .zip from the row above, open NinjaTrader 8 → Tools → Import → NinjaScript Add-On. Pick the .zip. Restart NT. Apply the indicator from the indicator dialog on any chart.'
		},
		{
			id: 'i2',
			title: 'TradingView Pine v6 — copy & paste',
			content:
				'Open TradingView → Pine Editor (bottom panel). Paste the script from the .txt file. Click "Save" then "Add to chart". For best performance, set chart timeframe to 1m or 5m.'
		},
		{
			id: 'i3',
			title: 'ThinkOrSwim — import the study',
			content:
				'Open ThinkOrSwim → Studies → Edit Studies → Import. Pick the .ts file. Apply to your chart and enable alerts via right-click.'
		}
	];

	function copyKey(key: string) {
		navigator.clipboard?.writeText(key);
		toasts.success('License key copied.');
	}
</script>

<Seo title="My Indicators" noindex />

<header class="ph">
	<div>
		<p class="eyebrow">Library</p>
		<h2>Your installed tools.</h2>
		<p class="muted">Lifetime updates · always download the latest version below.</p>
	</div>
</header>

{#each owned as ind}
	<article class="ind-card">
		<header class="ind-h">
			<div class="ind-thumb" style:background={ind.media.posterColor}>
				<IconCpu size={28} />
			</div>
			<div class="ind-meta">
				<div class="ind-titles">
					<h3>{ind.name}</h3>
					<Badge variant="success" size="sm"
						>{#snippet children()}<IconCheck size={10} />Active{/snippet}</Badge
					>
				</div>
				<p class="muted">{ind.tagline}</p>
			</div>
			<Button variant="gold-outline" size="sm" href="/indicators/{ind.slug}">
				{#snippet iconLeft()}<IconBookmark size={14} />{/snippet}
				Public page
			</Button>
		</header>

		<section class="key-row">
			<div>
				<p class="kl"><IconKey size={12} /> License key</p>
				<p class="kk">{ind.licenseKey}</p>
			</div>
			<Button variant="ghost" size="sm" onclick={() => copyKey(ind.licenseKey)}>Copy key</Button>
		</section>

		<section class="dl-section">
			<header class="dl-h">
				<h4><IconDownload size={14} /> Downloads</h4>
				<span class="muted">Latest version · {ind.downloads[0]?.version}</span>
			</header>
			<table>
				<thead>
					<tr><th>Platform</th><th>Version</th><th>Released</th><th>Size</th><th></th></tr>
				</thead>
				<tbody>
					{#each ind.downloads as d}
						<tr>
							<td class="strong">{d.platform}</td>
							<td class="mono">{d.version}</td>
							<td class="muted">{d.date}</td>
							<td class="muted">{d.size}</td>
							<td class="right">
								<Button variant="gold-outline" size="sm">
									{#snippet iconLeft()}<IconDownload size={12} />{/snippet}
									Download
								</Button>
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</section>

		<section class="dl-section">
			<header class="dl-h">
				<h4><IconRefresh size={14} /> Install guides</h4>
			</header>
			<Accordion items={installGuide} />
		</section>
	</article>
{/each}

<style>
	.ph {
		margin-bottom: clamp(2rem, 4vw, 3rem);
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
	.ph h2 {
		font-family: var(--font-display);
		font-size: clamp(1.75rem, 3vw, 2.5rem);
		margin: var(--space-2) 0;
	}
	.muted {
		color: var(--ink-400);
		font-size: var(--text-sm);
		margin: 0;
	}

	.ind-card {
		padding: var(--space-6);
		background: var(--surface-1);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-xl);
		display: flex;
		flex-direction: column;
		gap: var(--space-5);
	}

	.ind-h {
		display: grid;
		grid-template-columns: 88px 1fr auto;
		gap: var(--space-4);
		align-items: center;
	}
	.ind-thumb {
		width: 88px;
		height: 88px;
		border-radius: var(--radius-lg);
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--gold-300);
	}
	.ind-titles {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}
	.ind-titles h3 {
		font-family: var(--font-display);
		font-size: var(--text-xl);
		margin: 0;
	}
	.ind-meta .muted {
		font-size: var(--text-sm);
		margin-top: var(--space-2);
	}

	.key-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-3);
		padding: var(--space-4);
		background: linear-gradient(135deg, rgba(232, 182, 96, 0.04), var(--surface-2));
		border: 1px solid var(--border-gold);
		border-radius: var(--radius-md);
	}
	.kl {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		font-size: var(--text-2xs);
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		color: var(--gold-400);
		font-weight: var(--weight-semibold);
		margin: 0;
	}
	.kk {
		font-family: var(--font-mono);
		font-size: var(--text-md);
		color: var(--ink-100);
		margin: 4px 0 0;
		letter-spacing: 0.05em;
	}

	.dl-section {
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
	}
	.dl-h {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}
	.dl-h h4 {
		font-family: var(--font-display);
		font-size: var(--text-md);
		margin: 0;
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
	}
	.dl-h h4 :global(svg) {
		color: var(--gold-400);
	}

	table {
		width: 100%;
		border-collapse: collapse;
	}
	th {
		text-align: left;
		padding: 0 var(--space-3) var(--space-2);
		font-size: var(--text-xs);
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		color: var(--ink-400);
		font-weight: var(--weight-semibold);
		border-bottom: 1px solid var(--border-default);
	}
	td {
		padding: var(--space-3);
		font-size: var(--text-sm);
		color: var(--ink-200);
		border-bottom: 1px solid var(--border-subtle);
		vertical-align: middle;
	}
	td.right {
		text-align: right;
	}
	td.strong {
		color: var(--ink-100);
		font-weight: var(--weight-medium);
	}
	td.mono {
		font-family: var(--font-mono);
		font-size: var(--text-xs);
		color: var(--ink-300);
	}
	td.muted {
		color: var(--ink-400);
	}
</style>
