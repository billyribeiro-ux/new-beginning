import type { PageServerLoad } from './$types';
import { db, schema } from '$lib/server/db/client.js';
import { desc } from 'drizzle-orm';

export const load: PageServerLoad = async () => {
	let recentLeads: (typeof schema.leads.$inferSelect)[] = [];
	let recentMessages: (typeof schema.contactMessages.$inferSelect)[] = [];
	try {
		recentLeads = await db
			.select()
			.from(schema.leads)
			.orderBy(desc(schema.leads.createdAt))
			.limit(5);
		recentMessages = await db
			.select()
			.from(schema.contactMessages)
			.orderBy(desc(schema.contactMessages.createdAt))
			.limit(5);
	} catch {
		// DB not migrated yet — render with empties
	}
	return { recentLeads, recentMessages };
};
