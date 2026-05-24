<script lang="ts">
	import type { PageData } from './$types';
	import { IconArrowLeft, IconDeviceFloppy, IconTrash } from '@tabler/icons-svelte';
	import Seo from '$lib/components/seo/Seo.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import Textarea from '$lib/components/ui/Textarea.svelte';
	import Select from '$lib/components/ui/Select.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Switch from '$lib/components/ui/Switch.svelte';
	import Badge from '$lib/components/ui/Badge.svelte';
	import { toasts } from '$lib/stores/toast.svelte.js';
	import { untrack } from 'svelte';
	import { resolve } from '$app/paths';

	let { data }: { data: PageData } = $props();
	const p = $derived(data.product);

	let name = $state(untrack(() => data.product.name));
	let slug = $state(untrack(() => data.product.slug));
	let kind = $state<string>(untrack(() => data.product.kind));
	let price = $state(untrack(() => String(data.product.priceCents)));
	let tagline = $state(untrack(() => data.product.tagline));
	let description = $state(untrack(() => data.product.description));
	let active = $state(true);

	function save() {
		toasts.success(`Saved “${name}”`, 'Phase 1 stub.');
	}
</script>

<Seo title="Admin · {p.name}" noindex />

<header class="ph">
	<a class="back" href={resolve('/admin/products')}><IconArrowLeft size={14} />Back to products</a>
	<div class="ph-row">
		<div>
			<p class="eyebrow">Catalog · Edit</p>
			<h2>{p.name} <Badge variant="outline" size="sm">{p.kind}</Badge></h2>
		</div>
		<div class="actions">
			<Button variant="ghost">
				{#snippet iconLeft()}<IconTrash size={14} />{/snippet}
				Delete
			</Button>
			<Button variant="primary" onclick={save}>
				{#snippet iconLeft()}<IconDeviceFloppy size={14} />{/snippet}
				Save changes
			</Button>
		</div>
	</div>
</header>

<form
	class="grid"
	onsubmit={(e) => {
		e.preventDefault();
		save();
	}}
>
	<section class="card">
		<h3>Details</h3>
		<div class="form-grid">
			<Input label="Display name" name="name" bind:value={name} />
			<Input label="Slug" name="slug" bind:value={slug} />
			<Select
				label="Kind"
				name="kind"
				bind:value={kind}
				options={[
					{ value: 'indicator', label: 'Indicator' },
					{ value: 'course', label: 'Course' }
				]}
			/>
			<Input
				label="Price (cents)"
				name="price"
				type="number"
				bind:value={price}
				inputmode="numeric"
			/>
		</div>
		<Input label="Tagline" name="tagline" bind:value={tagline} />
		<Textarea label="Description" name="description" bind:value={description} rows={8} />
	</section>

	<aside class="side">
		<div class="card">
			<h3>Status</h3>
			<Switch bind:checked={active} label="Active" description="Toggle to hide from storefront." />
			<dl class="meta">
				<div>
					<dt>ID</dt>
					<dd class="mono">{p.id}</dd>
				</div>
				<div>
					<dt>Rating</dt>
					<dd>{p.rating.value.toFixed(1)} / 5 · {p.rating.count} reviews</dd>
				</div>
				<div>
					<dt>Last updated</dt>
					<dd>May 12, 2026</dd>
				</div>
			</dl>
		</div>

		<div class="card">
			<h3>Highlights</h3>
			<ul class="hl">
				{#each p.highlights as h, i (h)}
					<li><strong>0{i + 1}</strong>{h}</li>
				{/each}
			</ul>
		</div>
	</aside>
</form>

<style>
	.ph {
		margin-bottom: clamp(2rem, 4vw, 3rem);
	}
	.back {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		font-size: var(--text-xs);
		color: var(--ink-400);
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		text-decoration: none;
		font-weight: var(--weight-semibold);
	}
	.back:hover {
		color: var(--gold-300);
	}
	.ph-row {
		display: flex;
		justify-content: space-between;
		align-items: flex-end;
		gap: var(--space-4);
		flex-wrap: wrap;
		margin-top: var(--space-3);
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
		margin: var(--space-2) 0 0;
		display: inline-flex;
		align-items: center;
		gap: var(--space-3);
	}
	.actions {
		display: flex;
		gap: var(--space-2);
	}

	.grid {
		display: grid;
		grid-template-columns: 1fr;
		gap: var(--space-5);
	}
	@media (min-width: 1280px) {
		.grid {
			grid-template-columns: 1.5fr 1fr;
		}
	}

	.card {
		padding: var(--space-6);
		background: var(--surface-1);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-lg);
		display: flex;
		flex-direction: column;
		gap: var(--space-5);
	}
	.card h3 {
		font-family: var(--font-display);
		font-size: var(--text-md);
		margin: 0;
		padding-bottom: var(--space-3);
		border-bottom: 1px solid var(--border-default);
	}
	.form-grid {
		display: grid;
		grid-template-columns: 1fr;
		gap: var(--space-4);
	}
	@media (min-width: 640px) {
		.form-grid {
			grid-template-columns: 1fr 1fr;
		}
	}

	.side {
		display: flex;
		flex-direction: column;
		gap: var(--space-5);
	}
	.meta {
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
	}
	.meta > div {
		display: flex;
		justify-content: space-between;
		font-size: var(--text-xs);
	}
	.meta dt {
		color: var(--ink-400);
		text-transform: uppercase;
		letter-spacing: var(--tracking-wider);
	}
	.meta dd {
		margin: 0;
		color: var(--ink-100);
	}
	.meta .mono {
		font-family: var(--font-mono);
	}

	.hl {
		list-style: none;
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
	}
	.hl li {
		display: grid;
		grid-template-columns: 28px 1fr;
		gap: var(--space-3);
		font-size: var(--text-sm);
		color: var(--ink-200);
	}
	.hl strong {
		color: var(--gold-400);
		font-family: var(--font-mono);
		font-size: var(--text-xs);
		font-weight: var(--weight-semibold);
	}
</style>
