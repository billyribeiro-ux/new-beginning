<script lang="ts">
	import { IconArrowLeft, IconDeviceFloppy } from '@tabler/icons-svelte';
	import { resolve } from '$app/paths';
	import Seo from '$lib/components/seo/Seo.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import Textarea from '$lib/components/ui/Textarea.svelte';
	import Select from '$lib/components/ui/Select.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Switch from '$lib/components/ui/Switch.svelte';
	import { toasts } from '$lib/stores/toast.svelte.js';

	let name = $state('');
	let slug = $state('');
	let kind = $state('indicator');
	let price = $state('');
	let tagline = $state('');
	let description = $state('');
	let active = $state(true);

	function save() {
		if (!name || !slug || !price) {
			toasts.error('Name, slug and price are required.');
			return;
		}
		toasts.success(`Saved “${name}”`, 'Phase 1 stub — Drizzle write coming in Phase 2.');
	}
</script>

<Seo title="Admin · New product" noindex />

<header class="ph">
	<a class="back" href={resolve('/admin/products')}><IconArrowLeft size={14} />Back to products</a>
	<div class="ph-row">
		<div>
			<p class="eyebrow">Catalog · Create</p>
			<h2>New product</h2>
		</div>
		<div class="actions">
			<Button variant="ghost" href="/admin/products">Cancel</Button>
			<Button variant="primary" onclick={save}>
				{#snippet iconLeft()}<IconDeviceFloppy size={14} />{/snippet}
				Save product
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
		<h3>Basics</h3>
		<div class="form-grid">
			<Input
				label="Display name"
				name="name"
				bind:value={name}
				placeholder="e.g. Liquidity Hawk"
				required
			/>
			<Input
				label="URL slug"
				name="slug"
				bind:value={slug}
				placeholder="e.g. liquidity-hawk"
				required
			/>
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
				placeholder="99700"
				inputmode="numeric"
				required
			/>
		</div>
		<Input
			label="Tagline"
			name="tagline"
			bind:value={tagline}
			placeholder="One sentence that sells it."
		/>
		<Textarea
			label="Description"
			name="description"
			bind:value={description}
			placeholder="Lorem ipsum dolor sit amet…"
			rows={6}
		/>
	</section>

	<aside class="side">
		<div class="card">
			<h3>Status</h3>
			<Switch
				bind:checked={active}
				label="Active"
				description="When off, the product is hidden from the storefront."
			/>
		</div>
		<div class="card">
			<h3>SEO preview</h3>
			<div class="seo-prev">
				<p class="seo-url">
					tradeflextrading.com / {kind === 'indicator' ? 'indicators' : 'courses'} /
					<strong>{slug || 'product-slug'}</strong>
				</p>
				<p class="seo-title">{name || 'Product display name'}</p>
				<p class="seo-desc">
					{tagline || 'A short, plain-language tagline shows here in search results.'}
				</p>
			</div>
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
	@media (--bp-xl) {
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
	@media (--bp-md) {
		.form-grid {
			grid-template-columns: 1fr 1fr;
		}
	}

	.side {
		display: flex;
		flex-direction: column;
		gap: var(--space-5);
	}

	.seo-prev {
		padding: var(--space-4);
		background: var(--surface-0);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-md);
		font-family: 'Arial', sans-serif;
	}
	.seo-url {
		font-size: var(--text-xs);
		color: var(--success);
		margin: 0;
		word-break: break-all;
	}
	.seo-title {
		font-size: var(--text-lg);
		color: #6e8eff;
		margin: var(--space-2) 0 4px;
	}
	.seo-desc {
		font-size: var(--text-sm);
		color: var(--ink-300);
		margin: 0;
		line-height: var(--leading-relaxed);
	}
</style>
