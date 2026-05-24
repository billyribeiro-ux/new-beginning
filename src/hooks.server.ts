import type { Handle } from '@sveltejs/kit';

/**
 * Phase 1: no real auth. We attach a deterministic mock user so the
 * dashboard/admin routes can render against `event.locals.user`. The cookie
 * `tradeflex-mock-user` (if present) overrides the default role for demos.
 */
function mockUser(event: Parameters<Handle>[0]['event']): App.Locals['user'] {
	const cookieRole = event.cookies.get('tradeflex-mock-user');
	const role: 'member' | 'admin' = cookieRole === 'admin' ? 'admin' : 'member';
	return {
		id: 'usr_demo_001',
		email: role === 'admin' ? 'admin@tradeflextrading.com' : 'alex.morgan@example.com',
		name: role === 'admin' ? 'Admin Operator' : 'Alex Morgan',
		role
	};
}

export const handle: Handle = async ({ event, resolve }) => {
	event.locals.user = mockUser(event);

	const response = await resolve(event);

	// Security headers (production-ready baseline)
	response.headers.set('X-Content-Type-Options', 'nosniff');
	response.headers.set('Referrer-Policy', 'strict-origin-when-cross-origin');
	response.headers.set(
		'Permissions-Policy',
		'camera=(), microphone=(), geolocation=(), interest-cohort=()'
	);
	response.headers.set('X-Frame-Options', 'SAMEORIGIN');

	return response;
};
