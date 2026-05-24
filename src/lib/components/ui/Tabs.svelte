<script lang="ts">
	import type { Snippet } from 'svelte';

	type Tab = { id: string; label: string };
	type Props = {
		tabs: Tab[];
		value?: string;
		onchange?: (id: string) => void;
		panel?: Snippet<[string]>;
	};

	let { tabs, value = $bindable(), onchange, panel }: Props = $props();
	$effect(() => {
		if (!value && tabs[0]) value = tabs[0].id;
	});

	function select(id: string) {
		value = id;
		onchange?.(id);
	}
</script>

<div class="tabs">
	<div class="tablist" role="tablist">
		{#each tabs as t}
			<button
				type="button"
				role="tab"
				class="tab"
				class:is-active={value === t.id}
				aria-selected={value === t.id}
				onclick={() => select(t.id)}
			>
				{t.label}
			</button>
		{/each}
	</div>
	{#if panel && value}
		<div role="tabpanel" class="panel">
			{@render panel(value)}
		</div>
	{/if}
</div>

<style>
	.tabs {
		display: flex;
		flex-direction: column;
		gap: var(--space-5);
	}
	.tablist {
		display: inline-flex;
		gap: var(--space-1);
		padding: var(--space-1);
		background: var(--surface-1);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-full);
		width: max-content;
		max-width: 100%;
		overflow-x: auto;
		scrollbar-width: none;
	}
	.tablist::-webkit-scrollbar {
		display: none;
	}
	.tab {
		padding: var(--space-2) var(--space-5);
		font-size: var(--text-sm);
		font-weight: var(--weight-medium);
		color: var(--ink-300);
		border-radius: var(--radius-full);
		white-space: nowrap;
		transition: all var(--dur-base) var(--ease-out);
	}
	.tab:hover {
		color: var(--ink-100);
	}
	.tab.is-active {
		color: var(--surface-0);
		background: var(--gradient-gold);
		font-weight: var(--weight-semibold);
		box-shadow: 0 4px 12px rgba(212, 162, 76, 0.32);
	}
</style>
