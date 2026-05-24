import type { PageServerLoad } from './$types';
import { db, schema } from '$lib/server/db/client.js';
import { desc } from 'drizzle-orm';

export const load: PageServerLoad = async () => {
	let messages: (typeof schema.contactMessages.$inferSelect)[] = [];
	try {
		messages = await db
			.select()
			.from(schema.contactMessages)
			.orderBy(desc(schema.contactMessages.createdAt))
			.limit(200);
	} catch {
		/* DB not migrated */
	}
	return { messages };
};
