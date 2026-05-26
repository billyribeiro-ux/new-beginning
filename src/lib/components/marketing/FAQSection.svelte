<script lang="ts">
	import type { FAQ } from '$lib/data/faqs.js';
	import Accordion from '$lib/components/ui/Accordion.svelte';
	import JsonLd from '$lib/components/seo/JsonLd.svelte';
	import { faqLd } from '$lib/utils/jsonld.js';
	import { fadeUp } from '$lib/animations/attachments.js';

	type Props = { items: FAQ[]; emitJsonLd?: boolean };
	let { items, emitJsonLd = true }: Props = $props();

	const accordionItems = $derived(
		items.map((f, i) => ({ id: `faq-${i}`, title: f.q, content: f.a }))
	);
</script>

<div class="faq" {@attach fadeUp({ y: 18 })}>
	<Accordion items={accordionItems} defaultOpenId={accordionItems[0]?.id} />
</div>

{#if emitJsonLd}
	<JsonLd data={faqLd(items.map((f) => ({ q: f.q, a: f.a })))} />
{/if}

<style>
	.faq {
		max-width: 880px;
		margin-inline: auto;
	}
</style>
