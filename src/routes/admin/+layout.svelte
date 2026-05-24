<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { LayoutData } from './$types';
	import AdminSidebar from '$lib/components/admin/AdminSidebar.svelte';
	import AdminHeader from '$lib/components/admin/AdminHeader.svelte';

	type Props = { data: LayoutData; children: Snippet };
	let { data, children }: Props = $props();
</script>

<div class="shell">
	<AdminSidebar />
	<div class="col">
		{#if data.user}
			<AdminHeader user={data.user} />
		{/if}
		<main class="content">
			{@render children()}
		</main>
	</div>
</div>

<style>
	.shell {
		display: grid;
		grid-template-columns: auto 1fr;
		min-height: 100dvh;
		background: var(--surface-0);
	}
	.col {
		display: flex;
		flex-direction: column;
		min-width: 0;
	}
	.content {
		flex: 1;
		padding: clamp(1.5rem, 3vw, 2.5rem);
	}
	@media (max-width: 1023px) {
		.shell {
			grid-template-columns: 1fr;
		}
	}
</style>
