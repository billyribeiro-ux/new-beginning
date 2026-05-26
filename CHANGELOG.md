# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-05-24

Security and quality release. Closes a set of access-control gaps in the
admin/dashboard surfaces, hardens session cookies, introduces basic rate
limiting and CSP, and clears the long tail of accessibility and lint debt
that had accumulated during the Phase 1 scaffold.

### Security

- Admin layout now enforces a role guard via `error(403)`. Previously any
  authenticated (or even unauthenticated, see below) user could read
  `/admin/*`, exposing all leads and contact messages.
  File: [src/routes/admin/+layout.server.ts](src/routes/admin/+layout.server.ts).
- Dashboard layout now redirects unauthenticated users to `/login` with a
  safe `redirectTo` query param.
  File: [src/routes/dashboard/+layout.server.ts](src/routes/dashboard/+layout.server.ts).
- Mock-auth cookie hardened: added `httpOnly`, `sameSite: 'lax'`, and
  `secure: !dev` on both login and signup actions. The cookie was
  previously readable by any client-side JavaScript.
  Files: [src/routes/login/+page.server.ts](src/routes/login/+page.server.ts),
  [src/routes/signup/+page.server.ts](src/routes/signup/+page.server.ts).
- `event.locals.user` now resolves to `null` when no auth cookie is
  present. The previous implementation synthesized a phantom `member`
  user for every request, which is what made the dashboard appear
  unprotected end-to-end.
  File: [src/hooks.server.ts](src/hooks.server.ts).
- Open-redirect-safe `redirectTo` handling on `/login` — only accepts
  paths starting with a single `/` (rejects `//evil.example`).
  File: [src/routes/login/+page.server.ts](src/routes/login/+page.server.ts).
- HSTS header (`max-age=63072000; includeSubDomains; preload`) emitted in
  production responses. Dev unchanged.
  File: [src/hooks.server.ts](src/hooks.server.ts).
- Content-Security-Policy enabled for production builds.
  `script-src`/`style-src`/`connect-src` are locked to `'self'` plus a
  Google Fonts allowlist. Dev intentionally skips CSP so Vite HMR and
  `eval` continue to work.
  File: [svelte.config.js](svelte.config.js).
- Primary-key generation for lead and contact-message rows switched from
  `Math.random` to `crypto.randomUUID`.
  Files: [src/routes/free-guide/+page.server.ts](src/routes/free-guide/+page.server.ts),
  [src/routes/contact/+page.server.ts](src/routes/contact/+page.server.ts).
- In-memory rate limiter added: 5 req/min on `/free-guide` (capture and
  subscribe) and 3 req/min on `/contact`. Exceeding the budget returns
  HTTP 429 with a `retry-after` (seconds) hint.
  New file: [src/lib/server/rateLimit.ts](src/lib/server/rateLimit.ts).
- Inline script in `app.html` removed. It was a no-op that re-set
  `data-theme`, which is already present on `<html>` at SSR time.
  Removing it eliminates the need for a CSP nonce.
  File: [src/app.html](src/app.html).

### Added

- Indexes on `leads.created_at` and `contact_messages.created_at`.
  `EXPLAIN QUERY PLAN` now reports
  `SCAN ... USING INDEX leads_created_at_idx` instead of a full table
  scan.
  Files: [src/lib/server/db/schema.ts](src/lib/server/db/schema.ts),
  [drizzle/0001_mysterious_retro_girl.sql](drizzle/0001_mysterious_retro_girl.sql).
- Focus-trap utility for accessible dialogs.
  New file: [src/lib/utils/focusTrap.ts](src/lib/utils/focusTrap.ts).
- Rate-limiter utility (see Security).
- ESLint dev dependency `@eslint/js`. It was referenced by the ESLint
  config but missing from `package.json`, so the lint pipeline had never
  successfully run before this release.

### Changed

- `prefersReducedMotion` store now uses Svelte's built-in `MediaQuery`
  from `svelte/reactivity`. Eliminates the manual `addEventListener`
  call that had no matching cleanup.
  File: [src/lib/stores/prefersReducedMotion.svelte.ts](src/lib/stores/prefersReducedMotion.svelte.ts).
- All static `href="/path"` links across 26 `.svelte` files now use
  `href={resolve('/path')}` via SvelteKit's `$app/paths`. Satisfies the
  `svelte/no-navigation-without-resolve` lint rule and future-proofs the
  app for base-path deployments.
- `pnpm-lock.yaml` added to `.prettierignore` (was producing spurious
  format warnings on every prettier run).

### Fixed

- Memory leak in `prefersReducedMotion` store: the `MediaQueryList`
  listener was never removed on teardown.
- Focus trap on `Modal` and `CartDrawer`: Tab and Shift+Tab now cycle
  inside the dialog; previously focus could escape to interactive
  elements on the page below.
  Files: [src/lib/components/ui/Modal.svelte](src/lib/components/ui/Modal.svelte),
  [src/lib/components/layout/CartDrawer.svelte](src/lib/components/layout/CartDrawer.svelte).
- Navbar active link now exposes `aria-current="page"` for screen
  readers.
  File: [src/lib/components/layout/Navbar.svelte](src/lib/components/layout/Navbar.svelte).
- 23 unused-import warnings cleared across the route tree.
- 3 `{#each}` blocks were missing a key expression and are now keyed
  properly.
- Pre-existing ESLint config error: `@eslint/js` was not declared in
  `package.json`. The lint pipeline now runs end-to-end.
- Dev-server breakage from the initial CSP attempt — CSP no longer
  applies in dev mode, so Vite HMR, `eval`, and Google Fonts all work
  again.

## [0.1.0] - 2026-05-23

- Initial Phase 1 scaffold (see STACK.md).
