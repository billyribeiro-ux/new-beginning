import { z } from 'zod';

export const emailSchema = z
	.string()
	.min(1, 'Email is required')
	.max(320, 'Email is too long')
	.email('Please enter a valid email');

export const passwordSchema = z
	.string()
	.min(8, 'Password must be at least 8 characters')
	.max(128, 'Password is too long')
	.refine((v) => /[A-Z]/.test(v), 'Add at least one uppercase letter')
	.refine((v) => /[a-z]/.test(v), 'Add at least one lowercase letter')
	.refine((v) => /[0-9]/.test(v), 'Add at least one number');

export const nameSchema = z.string().min(1, 'Name is required').max(80, 'Name is too long');

export const leadCaptureSchema = z.object({
	email: emailSchema,
	source: z.string().max(64).optional(),
	// honeypot — must be empty
	website: z.string().max(0, 'Bot detected').optional().or(z.literal(''))
});

export const contactSchema = z.object({
	name: nameSchema,
	email: emailSchema,
	subject: z.string().min(2, 'Subject is required').max(160),
	body: z.string().min(10, 'Message is too short').max(4000),
	website: z.string().max(0).optional().or(z.literal(''))
});

export const loginSchema = z.object({
	email: emailSchema,
	password: z.string().min(1, 'Password is required'),
	remember: z.coerce.boolean().optional()
});

export const signupSchema = z
	.object({
		name: nameSchema,
		email: emailSchema,
		password: passwordSchema,
		confirm: z.string(),
		terms: z.coerce.boolean().refine((v) => v === true, 'You must accept the terms')
	})
	.refine((v) => v.password === v.confirm, {
		path: ['confirm'],
		message: 'Passwords do not match'
	});

export const forgotPasswordSchema = z.object({ email: emailSchema });

export const resetPasswordSchema = z
	.object({
		token: z.string().min(1),
		password: passwordSchema,
		confirm: z.string()
	})
	.refine((v) => v.password === v.confirm, {
		path: ['confirm'],
		message: 'Passwords do not match'
	});

export type LeadCaptureInput = z.infer<typeof leadCaptureSchema>;
export type ContactInput = z.infer<typeof contactSchema>;
export type LoginInput = z.infer<typeof loginSchema>;
export type SignupInput = z.infer<typeof signupSchema>;

export function scorePassword(value: string): { score: 0 | 1 | 2 | 3 | 4; label: string } {
	let score = 0;
	if (value.length >= 8) score++;
	if (value.length >= 12) score++;
	if (/[A-Z]/.test(value) && /[a-z]/.test(value)) score++;
	if (/[0-9]/.test(value) && /[^A-Za-z0-9]/.test(value)) score++;
	const labels = ['Too weak', 'Weak', 'Fair', 'Strong', 'Excellent'] as const;
	const clamped = Math.min(4, score) as 0 | 1 | 2 | 3 | 4;
	return { score: clamped, label: labels[clamped] };
}
