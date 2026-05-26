import { sql } from 'drizzle-orm';
import { sqliteTable, text, integer, index } from 'drizzle-orm/sqlite-core';

/* ─────────────── Leads (email captures) ─────────────── */
export const leads = sqliteTable(
	'leads',
	{
		id: text('id').primaryKey(),
		email: text('email').notNull(),
		source: text('source').notNull().default('free-guide'),
		createdAt: integer('created_at', { mode: 'timestamp_ms' })
			.notNull()
			.default(sql`(unixepoch() * 1000)`)
	},
	(t) => [index('leads_created_at_idx').on(t.createdAt)]
);

/* ─────────────── Users (schema-only stub) ─────────────── */
export const users = sqliteTable('users', {
	id: text('id').primaryKey(),
	email: text('email').notNull().unique(),
	name: text('name').notNull(),
	passwordHash: text('password_hash'),
	avatarUrl: text('avatar_url'),
	role: text('role', { enum: ['member', 'admin'] })
		.notNull()
		.default('member'),
	createdAt: integer('created_at', { mode: 'timestamp_ms' })
		.notNull()
		.default(sql`(unixepoch() * 1000)`)
});

/* ─────────────── Products (indicators + courses) ─────────────── */
export const products = sqliteTable('products', {
	id: text('id').primaryKey(),
	slug: text('slug').notNull().unique(),
	kind: text('kind', { enum: ['indicator', 'course'] }).notNull(),
	name: text('name').notNull(),
	tagline: text('tagline').notNull(),
	description: text('description').notNull(),
	priceCents: integer('price_cents').notNull(),
	active: integer('active', { mode: 'boolean' }).notNull().default(true),
	createdAt: integer('created_at', { mode: 'timestamp_ms' })
		.notNull()
		.default(sql`(unixepoch() * 1000)`)
});

/* ─────────────── Subscription plans ─────────────── */
export const subscriptionPlans = sqliteTable('subscription_plans', {
	id: text('id').primaryKey(),
	slug: text('slug').notNull().unique(),
	name: text('name').notNull(),
	cadence: text('cadence', { enum: ['monthly', 'quarterly', 'annual'] }).notNull(),
	priceCents: integer('price_cents').notNull(),
	savingsPct: integer('savings_pct').notNull().default(0)
});

/* ─────────────── Cart (Phase-2-ready scaffold) ─────────────── */
export const cartSessions = sqliteTable('cart_sessions', {
	id: text('id').primaryKey(),
	sessionToken: text('session_token').notNull().unique(),
	userId: text('user_id').references(() => users.id),
	createdAt: integer('created_at', { mode: 'timestamp_ms' })
		.notNull()
		.default(sql`(unixepoch() * 1000)`)
});

export const cartItems = sqliteTable('cart_items', {
	id: text('id').primaryKey(),
	sessionId: text('session_id')
		.notNull()
		.references(() => cartSessions.id, { onDelete: 'cascade' }),
	productId: text('product_id').references(() => products.id),
	planId: text('plan_id').references(() => subscriptionPlans.id),
	quantity: integer('quantity').notNull().default(1)
});

/* ─────────────── Contact form messages ─────────────── */
export const contactMessages = sqliteTable(
	'contact_messages',
	{
		id: text('id').primaryKey(),
		name: text('name').notNull(),
		email: text('email').notNull(),
		subject: text('subject').notNull(),
		body: text('body').notNull(),
		createdAt: integer('created_at', { mode: 'timestamp_ms' })
			.notNull()
			.default(sql`(unixepoch() * 1000)`)
	},
	(t) => [index('contact_messages_created_at_idx').on(t.createdAt)]
);

export type Lead = typeof leads.$inferSelect;
export type NewLead = typeof leads.$inferInsert;
export type User = typeof users.$inferSelect;
export type ProductRow = typeof products.$inferSelect;
export type SubscriptionPlanRow = typeof subscriptionPlans.$inferSelect;
export type ContactMessage = typeof contactMessages.$inferSelect;
