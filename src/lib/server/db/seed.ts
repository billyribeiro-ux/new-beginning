import { createClient } from '@libsql/client';
import { drizzle } from 'drizzle-orm/libsql';
import { readFileSync } from 'node:fs';
import * as schema from './schema.js';
import { INDICATORS, COURSES } from '../../data/products.js';
import { DAY_TRADING_PLANS } from '../../data/plans.js';

// Load .env manually (no $env in CLI context)
try {
	const env = readFileSync('.env', 'utf8');
	for (const line of env.split('\n')) {
		const m = line.match(/^([A-Z_][A-Z0-9_]*)=["']?([^"'\n]+)["']?/);
		if (m && m[1] && !process.env[m[1]]) process.env[m[1]] = m[2];
	}
} catch {
	/* no .env file */
}

const url = process.env.DATABASE_URL ?? 'file:./drizzle/local.db';
const client = createClient({ url });
const db = drizzle(client, { schema });

async function seed() {
	console.log('🌱  Seeding TradeFlex Trading database…');

	for (const p of [...INDICATORS, ...COURSES]) {
		await db
			.insert(schema.products)
			.values({
				id: p.id,
				slug: p.slug,
				kind: p.kind,
				name: p.name,
				tagline: p.tagline,
				description: p.description,
				priceCents: p.priceCents
			})
			.onConflictDoNothing();
		console.log(`  · ${p.kind.padEnd(9)} ${p.slug}`);
	}

	for (const pl of DAY_TRADING_PLANS) {
		await db
			.insert(schema.subscriptionPlans)
			.values({
				id: pl.id,
				slug: pl.slug,
				name: pl.name,
				cadence: pl.cadence,
				priceCents: pl.priceCents,
				savingsPct: pl.savingsPct ?? 0
			})
			.onConflictDoNothing();
		console.log(`  · plan      ${pl.slug}`);
	}

	console.log('✅  Done.');
	process.exit(0);
}

seed().catch((err) => {
	console.error(err);
	process.exit(1);
});
