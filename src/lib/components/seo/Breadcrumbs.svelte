<script lang="ts">
	import IconChevronRight from '@tabler/icons-svelte/icons/chevron-right';
	import JsonLd from './JsonLd.svelte';
	import { breadcrumbLd } from '$lib/utils/jsonld.js';
	import { resolve } from '$app/paths';

	type Crumb = { label: string; href: string };
	type Props = { items: Crumb[] };
	let { items }: Props = $props();
</script>

<nav class="breadcrumbs" aria-label="Breadcrumb">
	<ol>
		{#each items as crumb, i (crumb.href)}
			<li>
				{#if i === items.length - 1}
					<span aria-current="page">{crumb.label}</span>
				{:else}
					<a href={resolve(crumb.href as unknown as '/')}>{crumb.label}</a>
					<IconChevronRight size={12} class="sep" />
				{/if}
			</li>
		{/each}
	</ol>
</nav>

<JsonLd data={breadcrumbLd(items.map((c) => ({ name: c.label, url: c.href })))} />

<style>
	.breadcrumbs {
		font-size: var(--text-xs);
		color: var(--ink-400);
	}
	ol {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--space-2);
		list-style: none;
	}
	li {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
	}
	a {
		color: var(--ink-300);
		transition: color var(--dur-fast) var(--ease-out);
	}
	a:hover {
		color: var(--gold-300);
	}
	[aria-current='page'] {
		color: var(--ink-100);
		font-weight: var(--weight-medium);
	}
	:global(.sep) {
		color: var(--ink-500);
	}
</style>
