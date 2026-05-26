<script lang="ts">
	import type { Snippet } from 'svelte';
	import IconInbox from '@tabler/icons-svelte/icons/inbox';

	type IconLike = typeof IconInbox;
	type Props = {
		icon?: IconLike;
		title: string;
		description?: string;
		actions?: Snippet;
	};
	let { icon: Icon = IconInbox, title, description, actions }: Props = $props();
</script>

<div class="empty">
	<span class="ic"><Icon size={36} /></span>
	<h3>{title}</h3>
	{#if description}<p>{description}</p>{/if}
	{#if actions}<div class="actions">{@render actions()}</div>{/if}
</div>

<style>
	.empty {
		display: flex;
		flex-direction: column;
		align-items: center;
		text-align: center;
		gap: var(--space-3);
		padding: clamp(3rem, 8vw, 6rem) var(--space-6);
		background: var(--surface-1);
		border: 1px dashed var(--border-default);
		border-radius: var(--radius-xl);
	}
	.ic {
		width: 80px;
		height: 80px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		background: var(--surface-2);
		border-radius: var(--radius-full);
		color: var(--ink-400);
	}
	h3 {
		font-family: var(--font-display);
		font-size: var(--text-xl);
		margin: 0;
	}
	p {
		max-width: 48ch;
		color: var(--ink-400);
		font-size: var(--text-sm);
		margin: 0;
	}
	.actions {
		margin-top: var(--space-3);
		display: flex;
		gap: var(--space-3);
		flex-wrap: wrap;
		justify-content: center;
	}
</style>
