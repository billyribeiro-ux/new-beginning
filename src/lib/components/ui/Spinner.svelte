<script lang="ts">
	type Props = { size?: number; label?: string };
	let { size = 20, label = 'Loading' }: Props = $props();
</script>

<span class="spinner" style:--sz="{size}px" role="status" aria-label={label}>
	<span class="ring" aria-hidden="true"></span>
</span>

<style>
	.spinner {
		display: inline-flex;
		width: var(--sz);
		height: var(--sz);
	}
	.ring {
		width: 100%;
		height: 100%;
		border: 2px solid var(--surface-3);
		border-top-color: var(--gold-400);
		border-radius: var(--radius-full);
		animation: spin 0.7s linear infinite;
	}
</style>
