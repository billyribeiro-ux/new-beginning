<script lang="ts">
	import { IconShoppingBagPlus, IconCheck } from '@tabler/icons-svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import { cart, type CartLine } from '$lib/stores/cart.svelte.js';
	import { toasts } from '$lib/stores/toast.svelte.js';

	type Props = {
		line: Omit<CartLine, 'quantity'>;
		quantity?: number;
		variant?: 'primary' | 'secondary' | 'gold-outline';
		size?: 'sm' | 'md' | 'lg' | 'xl';
		fullWidth?: boolean;
		openOnAdd?: boolean;
	};

	let {
		line,
		quantity = 1,
		variant = 'primary',
		size = 'lg',
		fullWidth = false,
		openOnAdd = true
	}: Props = $props();

	let justAdded = $state(false);

	function add() {
		cart.add({ ...line, quantity });
		justAdded = true;
		toasts.success(`${line.name} added to cart`);
		setTimeout(() => (justAdded = false), 1600);
		if (openOnAdd) setTimeout(() => cart.open(), 240);
	}
</script>

<Button {variant} {size} {fullWidth} onclick={add}>
	{#snippet iconLeft()}
		{#if justAdded}<IconCheck size={18} />{:else}<IconShoppingBagPlus size={18} />{/if}
	{/snippet}
	{justAdded ? 'Added to cart' : 'Add to cart'}
</Button>
