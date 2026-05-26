// scripts/dump-catalog.ts
//
// BACKEND.md §1.6 + §21 row 2: catalog snapshot pipeline.
//
// PR #2 implementation note (see BACKEND_NOTES.md):
//   The frontend `src/lib/data/products.ts` imports Tabler icon *components*
//   which Node's tsx cannot resolve via the package's `exports` map. A full
//   auto-extraction is therefore deferred to a future PR (split data-only
//   constants into a separate file, OR build via vite/esbuild with a stub
//   plugin). For now the catalog snapshot at `backend/seeds/catalog.json`
//   is hand-authored and reviewed in PRs.
//
// This script's job today is **drift detection**: parse the two TS source
// files as text, extract every (slug, priceCents) pair via regex, and assert
// that the JSON snapshot agrees. Run via `pnpm run dump-catalog`.

import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = join(__dirname, '..');
const PRODUCTS_TS = join(ROOT, 'src', 'lib', 'data', 'products.ts');
const PLANS_TS = join(ROOT, 'src', 'lib', 'data', 'plans.ts');
const CATALOG_JSON = join(ROOT, 'backend', 'seeds', 'catalog.json');

interface CatalogProduct {
	slug: string;
	price_cents: number;
}
interface CatalogPlan {
	slug: string;
	price_cents: number;
}

interface Catalog {
	indicators: CatalogProduct[];
	courses: CatalogProduct[];
	plans: CatalogPlan[];
}

/** Extract `slug: '...'` followed by `priceCents: N` pairs from a TS source. */
function extractSlugPricePairs(ts: string): Map<string, number> {
	const out = new Map<string, number>();
	// One object per regex match. Greedy enough to span the few lines between
	// `slug:` and `priceCents:`; permissive on whitespace.
	const re = /slug:\s*['"]([^'"]+)['"][\s\S]*?priceCents:\s*(\d+)/g;
	for (const m of ts.matchAll(re)) {
		out.set(m[1]!, Number(m[2]!));
	}
	return out;
}

const productsTs = readFileSync(PRODUCTS_TS, 'utf8');
const plansTs = readFileSync(PLANS_TS, 'utf8');
const catalog: Catalog = JSON.parse(readFileSync(CATALOG_JSON, 'utf8'));

const expected = new Map<string, number>([
	...extractSlugPricePairs(productsTs),
	...extractSlugPricePairs(plansTs)
]);

const actual = new Map<string, number>([
	...catalog.indicators.map((p) => [p.slug, p.price_cents] as const),
	...catalog.courses.map((p) => [p.slug, p.price_cents] as const),
	...catalog.plans.map((p) => [p.slug, p.price_cents] as const)
]);

const errors: string[] = [];

for (const [slug, ePrice] of expected.entries()) {
	const aPrice = actual.get(slug);
	if (aPrice === undefined) {
		errors.push(`source TS has slug='${slug}' (${ePrice}c) but catalog.json is missing it`);
	} else if (aPrice !== ePrice) {
		errors.push(`drift on slug='${slug}': source TS=${ePrice}c, catalog.json=${aPrice}c`);
	}
}
for (const slug of actual.keys()) {
	if (!expected.has(slug)) {
		errors.push(`catalog.json has slug='${slug}' but source TS does not`);
	}
}

if (errors.length) {
	console.error(`catalog drift detected (${errors.length} item${errors.length === 1 ? '' : 's'}):`);
	for (const e of errors) console.error(`  - ${e}`);
	process.exit(1);
}

console.log(
	`catalog.json in sync: ${expected.size} slug${expected.size === 1 ? '' : 's'} verified ` +
		`(${catalog.indicators.length} indicators, ${catalog.courses.length} courses, ${catalog.plans.length} plans)`
);
