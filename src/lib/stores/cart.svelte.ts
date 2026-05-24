import { browser } from '$app/environment';

export type CartLineKind = 'product' | 'plan';

export interface CartLine {
	id: string;
	kind: CartLineKind;
	slug: string;
	name: string;
	subtitle?: string;
	priceCents: number;
	quantity: number;
	maxQuantity?: number;
}

const STORAGE_KEY = 'tradeflex:cart:v1';

function loadFromStorage(): CartLine[] {
	if (!browser) return [];
	try {
		const raw = localStorage.getItem(STORAGE_KEY);
		if (!raw) return [];
		const parsed = JSON.parse(raw);
		if (!Array.isArray(parsed)) return [];
		return parsed.filter(
			(l): l is CartLine =>
				typeof l?.id === 'string' &&
				typeof l?.name === 'string' &&
				Number.isFinite(l?.priceCents) &&
				Number.isFinite(l?.quantity)
		);
	} catch {
		return [];
	}
}

class CartStore {
	lines = $state<CartLine[]>([]);
	isOpen = $state(false);
	pulseToken = $state(0); // increments on add → animates badge

	count = $derived(this.lines.reduce((n, l) => n + l.quantity, 0));
	subtotalCents = $derived(this.lines.reduce((n, l) => n + l.priceCents * l.quantity, 0));
	isEmpty = $derived(this.lines.length === 0);

	constructor() {
		if (browser) {
			this.lines = loadFromStorage();
			$effect.root(() => {
				$effect(() => {
					try {
						localStorage.setItem(STORAGE_KEY, JSON.stringify(this.lines));
					} catch {
						/* quota or private mode — ignore */
					}
				});
			});
		}
	}

	add(line: Omit<CartLine, 'quantity'> & { quantity?: number }) {
		const qty = line.quantity ?? 1;
		const existing = this.lines.find((l) => l.id === line.id);
		if (existing) {
			existing.quantity = Math.min(existing.quantity + qty, existing.maxQuantity ?? 99);
		} else {
			this.lines = [
				...this.lines,
				{
					...line,
					quantity: Math.min(qty, line.maxQuantity ?? 99)
				}
			];
		}
		this.pulseToken++;
	}

	remove(id: string) {
		this.lines = this.lines.filter((l) => l.id !== id);
	}

	setQuantity(id: string, qty: number) {
		if (qty <= 0) return this.remove(id);
		this.lines = this.lines.map((l) =>
			l.id === id ? { ...l, quantity: Math.min(qty, l.maxQuantity ?? 99) } : l
		);
	}

	clear() {
		this.lines = [];
	}

	open() {
		this.isOpen = true;
	}

	close() {
		this.isOpen = false;
	}

	toggle() {
		this.isOpen = !this.isOpen;
	}
}

export const cart = new CartStore();
