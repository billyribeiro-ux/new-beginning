/**
 * Currency helpers. All prices stored as integer cents to avoid float drift.
 */

export function formatPrice(cents: number, currency = 'USD', locale = 'en-US'): string {
	return new Intl.NumberFormat(locale, {
		style: 'currency',
		currency,
		maximumFractionDigits: cents % 100 === 0 ? 0 : 2
	}).format(cents / 100);
}

export function formatPriceParts(cents: number): {
	whole: string;
	fraction: string;
	symbol: string;
} {
	const parts = new Intl.NumberFormat('en-US', {
		style: 'currency',
		currency: 'USD',
		minimumFractionDigits: 2
	}).formatToParts(cents / 100);

	let whole = '';
	let fraction = '';
	let symbol = '$';
	for (const p of parts) {
		if (p.type === 'integer' || p.type === 'group') whole += p.value;
		else if (p.type === 'fraction') fraction = p.value;
		else if (p.type === 'currency') symbol = p.value;
	}
	return { whole, fraction, symbol };
}

export function discountPct(originalCents: number, finalCents: number): number {
	if (originalCents <= 0 || finalCents >= originalCents) return 0;
	return Math.round(((originalCents - finalCents) / originalCents) * 100);
}

export function monthlyEquivalent(cents: number, months: number): number {
	return Math.round(cents / months);
}
