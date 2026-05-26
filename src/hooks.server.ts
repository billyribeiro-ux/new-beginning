import { dev } from '$app/environment';
import type { Handle } from '@sveltejs/kit';

/**
 * Phase 1: no real auth. The cookie `tradeflex-mock-user` is set by the
 * /login and /signup actions and read here to render dashboard pages.
 * A missing cookie means "not signed in" — `event.locals.user` is `null`.
 *
 * SECURITY: the cookie value is attacker-controlled (anyone can paste any
 * string into DevTools → Application → Cookies). It must NOT carry the role.
 * Until real auth lands, every authenticated user is a `member`; admin
 * elevation is impossible from the public surface. The `/admin` route is
 * therefore unreachable in Phase 1, which is the desired safe default.
 */
function mockUser(event: Parameters<Handle>[0]['event']): App.Locals['user'] {
	const cookieValue = event.cookies.get('tradeflex-mock-user');
	if (cookieValue !== 'member') return null;
	return {
		id: 'usr_demo_001',
		email: 'alex.morgan@example.com',
		name: 'Alex Morgan',
		role: 'member'
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
	if (!dev) {
		response.headers.set(
			'Strict-Transport-Security',
			'max-age=63072000; includeSubDomains; preload'
		);
	}

	return response;
};
