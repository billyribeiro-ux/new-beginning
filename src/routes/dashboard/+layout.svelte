<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { LayoutData } from './$types';
	import DashboardSidebar from '$lib/components/dashboard/DashboardSidebar.svelte';
	import DashboardHeader from '$lib/components/dashboard/DashboardHeader.svelte';

	type Props = { data: LayoutData; children: Snippet };
	let { data, children }: Props = $props();
</script>

<div class="shell">
	<DashboardSidebar />
	<div class="col">
		{#if data.user}
			<DashboardHeader user={data.user} />
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
	@media (--bp-lg-down) {
		.shell {
			grid-template-columns: 1fr;
		}
	}
</style>
