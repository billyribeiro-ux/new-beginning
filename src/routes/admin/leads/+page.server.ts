import type { PageServerLoad } from './$types';
import { db, schema } from '$lib/server/db/client.js';
import { desc } from 'drizzle-orm';

export const load: PageServerLoad = async () => {
	let leads: (typeof schema.leads.$inferSelect)[] = [];
	try {
		leads = await db.select().from(schema.leads).orderBy(desc(schema.leads.createdAt)).limit(200);
	} catch {
		/* DB not migrated yet */
	}
	return { leads };
};
