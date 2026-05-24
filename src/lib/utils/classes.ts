/** Tiny conditional class joiner. */
export type ClassValue =
	| string
	| number
	| false
	| null
	| undefined
	| Record<string, unknown>
	| ClassValue[];

export function cx(...values: ClassValue[]): string {
	const out: string[] = [];
	for (const v of values) {
		if (!v) continue;
		if (typeof v === 'string' || typeof v === 'number') {
			out.push(String(v));
		} else if (Array.isArray(v)) {
			const inner = cx(...v);
			if (inner) out.push(inner);
		} else if (typeof v === 'object') {
			for (const [k, val] of Object.entries(v)) if (val) out.push(k);
		}
	}
	return out.join(' ');
}
