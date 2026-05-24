<script lang="ts">
	import type { Snippet } from 'svelte';
	import { untrack } from 'svelte';
	import { IconPlus } from '@tabler/icons-svelte';
	import { slide } from 'svelte/transition';

	type Item = { id: string; title: string; content: string };

	type Props = {
		items: Item[];
		allowMultiple?: boolean;
		defaultOpenId?: string;
		titleSnippet?: Snippet<[Item]>;
		contentSnippet?: Snippet<[Item]>;
	};

	let {
		items,
		allowMultiple = false,
		defaultOpenId,
		titleSnippet,
		contentSnippet
	}: Props = $props();
	let openIds = $state<Set<string>>(untrack(() => new Set(defaultOpenId ? [defaultOpenId] : [])));

	function toggle(id: string) {
		const next = new Set(allowMultiple ? openIds : []);
		if (openIds.has(id)) {
			next.delete(id);
		} else {
			next.add(id);
		}
		openIds = next;
	}
</script>

<div class="accordion">
	{#each items as item (item.id)}
		{@const isOpen = openIds.has(item.id)}
		<div class="item" class:is-open={isOpen}>
			<button
				class="trigger"
				type="button"
				aria-expanded={isOpen}
				aria-controls="acc-{item.id}"
				onclick={() => toggle(item.id)}
			>
				<span class="title">
					{#if titleSnippet}{@render titleSnippet(item)}{:else}{item.title}{/if}
				</span>
				<span class="icon" aria-hidden="true">
					<IconPlus size={18} />
				</span>
			</button>
			{#if isOpen}
				<div id="acc-{item.id}" class="content" transition:slide={{ duration: 280 }}>
					<div class="content-inner">
						{#if contentSnippet}
							{@render contentSnippet(item)}
						{:else}
							<p>{item.content}</p>
						{/if}
					</div>
				</div>
			{/if}
		</div>
	{/each}
</div>

<style>
	.accordion {
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
	}
	.item {
		background: var(--surface-1);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-lg);
		overflow: hidden;
		transition:
			background var(--dur-base) var(--ease-out),
			border-color var(--dur-base) var(--ease-out);
	}
	.item.is-open {
		background: var(--surface-2);
		border-color: var(--border-gold);
	}
	.trigger {
		width: 100%;
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-4);
		padding: var(--space-5) var(--space-6);
		text-align: left;
		color: var(--ink-100);
		font-size: var(--text-md);
		font-weight: var(--weight-semibold);
		font-family: var(--font-display);
		line-height: var(--leading-snug);
		transition: color var(--dur-fast) var(--ease-out);
	}
	.trigger:hover {
		color: var(--gold-300);
	}
	.icon {
		flex-shrink: 0;
		width: 32px;
		height: 32px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		border-radius: var(--radius-full);
		background: var(--surface-3);
		color: var(--gold-400);
		transition: transform var(--dur-base) var(--ease-spring);
	}
	.item.is-open .icon {
		transform: rotate(45deg);
		background: var(--gradient-gold);
		color: var(--surface-0);
	}
	.content {
		overflow: hidden;
	}
	.content-inner {
		padding: 0 var(--space-6) var(--space-6);
		color: var(--ink-300);
		font-size: var(--text-sm);
		line-height: var(--leading-relaxed);
	}
</style>
