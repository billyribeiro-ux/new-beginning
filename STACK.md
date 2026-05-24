# TradeFlex Trading — Stack & Conventions (Phase 1)

This document is the canonical reference for **what** powers the site, **why** each
choice was made, and **how** the codebase is organized. Keep it current as the
project evolves.

---

## 1. Frameworks & languages

| Layer        | Tool                                     | Version (May 2026) | Notes                                                                                                                          |
| ------------ | ---------------------------------------- | ------------------ | ------------------------------------------------------------------------------------------------------------------------------ |
| Framework    | **SvelteKit**                            | `^2.61`            | Routing, SSR, form actions, endpoints, adapters.                                                                               |
| UI library   | **Svelte 5 (runes mode)**                | `^5.55`            | `$state`, `$derived`, `$effect`, `$props`, `{@attach}`, snippets. **No legacy slots, no `context="module"`, no `export let`.** |
| Language     | **TypeScript**                           | `^6.0`             | Strict mode (`strict`, `noUncheckedIndexedAccess`, `noImplicitOverride`, `verbatimModuleSyntax`, `isolatedModules`).           |
| Bundler      | **Vite**                                 | `^8.0`             | LightningCSS for `cssMinify`. ESM-only output.                                                                                 |
| Adapter      | `@sveltejs/adapter-auto`                 | `^7.0`             | Defer host decision. Swap to `adapter-node` / `adapter-vercel` for prod.                                                       |
| ORM          | **Drizzle ORM**                          | `^0.45`            | Type-safe SQL; SQLite via libsql client.                                                                                       |
| Database     | **SQLite (libsql)** via `@libsql/client` | `^0.17`            | Local file in dev. Drop-in to Turso for prod.                                                                                  |
| Icons        | `@tabler/icons-svelte`                   | `^3.44`            | **Sole icon source.** External SVG is forbidden and enforced via ESLint.                                                       |
| Animation    | **GSAP** + ScrollTrigger                 | `^3.15`            | Wrapped in Svelte 5 attachments for context-scoped cleanup. Honors `prefers-reduced-motion`.                                   |
| Validation   | **Zod**                                  | `^4.4`             | All form inputs (lead capture, contact, auth) are validated client + server.                                                   |
| Linting      | **ESLint 10** flat config                | `^10.4`            | `eslint-plugin-svelte` + `typescript-eslint`. Custom `no-restricted-imports` rule blocks `*.svg` imports.                      |
| Formatting   | **Prettier** + `prettier-plugin-svelte`  | `^3.8`             | Tabs · single quotes · `printWidth 100` · no trailing commas.                                                                  |
| Type checker | `svelte-check`                           | `^4.4`             | Zero errors, zero warnings is the gate.                                                                                        |

## 2. Project structure

```
src/
├─ app.html                       Pre-hydration shell — font preload, theme-color, skip link
├─ app.css                        Global stylesheet entry
├─ app.d.ts                       App.Locals (mocked user in Phase 1), App.PageData
├─ hooks.server.ts                Security headers, mock-user attachment to event.locals
├─ lib/
│  ├─ components/
│  │  ├─ layout/                  Navbar · Footer · AnnouncementBar · CartDrawer · AuthShell
│  │  ├─ ui/                      Button · Input · Textarea · Select · Checkbox · Switch · Badge · Modal · Tabs · Tooltip · Accordion · Spinner · Toast
│  │  ├─ marketing/               Hero · SectionHeading · FeatureGrid · StatBlock · TrustBar · TestimonialCarousel · FAQSection · CTABanner · LegalPage
│  │  ├─ commerce/                ProductCard · PricingCard · PriceTag · AddToCartButton
│  │  ├─ forms/                   EmailCaptureForm · ContactForm · AuthForm · PasswordField
│  │  ├─ seo/                     Seo · JsonLd · Breadcrumbs
│  │  ├─ media/                   LogoMark · LogoWordmark
│  │  ├─ dashboard/               DashboardSidebar · DashboardHeader · StatCard · ProgressBar · EmptyState
│  │  └─ admin/                   AdminSidebar · AdminHeader · DataTable · ChartCard (canvas, zero SVG)
│  ├─ stores/                     cart.svelte.ts · ui.svelte.ts · toast.svelte.ts · prefersReducedMotion.svelte.ts
│  ├─ server/db/                  schema.ts · client.ts · seed.ts (Drizzle, libsql)
│  ├─ data/                       products.ts · plans.ts · testimonials.ts · faqs.ts · navigation.ts
│  ├─ utils/                      money · classes · validators (Zod) · seo · jsonld · slug
│  ├─ animations/                 gsap.ts (lazy init) · attachments.ts (fadeUp, stagger, splitReveal, magnetic, numberTicker, parallax, cursorGlow)
│  └─ styles/                     tokens.css · reset.css · typography.css · utilities.css · animations.css
└─ routes/                        See § 3
static/
├─ pdf/options-greeks-guide.pdf   Lead-magnet artifact (placeholder)
└─ favicon.ico
drizzle/                          Generated SQL migrations + local SQLite (gitignored)
```

## 3. Routes (all built in Phase 1)

**Marketing**
`/` · `/subscription` · `/indicators` · `/indicators/[slug]` · `/courses` · `/courses/[slug]`
`/about` · `/contact` · `/cart`
`/free-guide` · `/free-guide/success`
`/legal/terms` · `/legal/privacy` · `/legal/refund`
`/sitemap.xml` · `/robots.txt`

**Auth (UI only — stubs)**
`/login` · `/signup` · `/forgot-password` · `/reset-password`

**Member dashboard (full UI, mock data + Drizzle reads)**
`/dashboard` (overview)
`/dashboard/courses` · `/dashboard/courses/[slug]` (player shell)
`/dashboard/indicators` (license keys + downloads)
`/dashboard/subscription` (change/pause/cancel flows)
`/dashboard/billing` (payment methods CRUD + invoices)
`/dashboard/profile` (profile · email · password · 2FA · sessions · export · delete)
`/dashboard/notifications` (granular prefs + DND + feed)
`/dashboard/downloads`

**Admin (full UI, Drizzle reads for leads/messages)**
`/admin` (KPI overview, revenue chart, recent leads/orders)
`/admin/products` · `/admin/products/new` · `/admin/products/[id]`
`/admin/courses` · `/admin/plans`
`/admin/orders` · `/admin/customers`
`/admin/leads` (live Drizzle) · `/admin/messages` (live Drizzle)
`/admin/settings` (general · SEO · email · integrations)

## 4. Design system

`src/lib/styles/tokens.css` is the **single source of truth** for colour, type,
spacing, radii, motion, breakpoints, container widths, and elevation. **All
components reference tokens** — never raw hex / px values.

- Theme: dark canvas (`--surface-0..4`) + gold/amber accent (`--gold-300..900`).
- Type: variable Fraunces (display) + Inter (body) + JetBrains Mono (numeric).
- Spacing: 4 px base (`--space-0..48`).
- Motion: easing curves (`--ease-out-expo`, `--ease-spring`, …) and durations
  (`--dur-fast..cinematic`). All keyframes collapse to instant on
  `prefers-reduced-motion: reduce`.
- Breakpoints (mobile-first, generous on big screens):
  `sm 480 · md 768 · lg 1024 · xl 1280 · 2xl 1536 · 3xl 1920 · 4xl 2560`.
  Container max-width scales up to **1840 px at 4xl**.

## 5. State management (Svelte 5 rune stores)

`*.svelte.ts` files export class instances using runes — no external store
library, no `writable`/`readable` legacy:

| Store                  | Owns                                                                                                |
| ---------------------- | --------------------------------------------------------------------------------------------------- |
| `cart`                 | line items, `count`, `subtotalCents`, drawer open state; persisted to `localStorage` via `$effect`. |
| `ui`                   | `mobileNavOpen`, `sidebarCollapsed` (persisted).                                                    |
| `toast`                | toast queue with auto-dismiss; rendered by `<Toast />` portal.                                      |
| `prefersReducedMotion` | reactive `matchMedia('(prefers-reduced-motion: reduce)')`.                                          |

## 6. Animation strategy

- `src/lib/animations/gsap.ts` lazily registers GSAP + ScrollTrigger client-side.
- `src/lib/animations/attachments.ts` exports `{@attach …}` helpers
  (`fadeUp`, `stagger`, `splitReveal`, `magnetic`, `numberTicker`, `parallax`,
  `cursorGlow`).
- Each attachment opens a `gsap.context(…, node)` and returns a cleanup that
  calls `ctx.revert()` — no orphan tweens on navigation.
- `prefersReducedMotion()` short-circuits each helper to an instant state.

## 7. Data & persistence (Phase 1)

- Catalog (`products`, `subscriptionPlans`) lives in `src/lib/data/*` for
  client-side rendering **and** is seeded into SQLite for parity.
- Two routes actually **write** to the DB:
  - `/free-guide` — `?/` and `?/subscribe` actions insert into `leads`.
  - `/contact` — default action inserts into `contactMessages`.
- Two admin routes **read live** from the DB:
  - `/admin/leads` — pulls from `leads`.
  - `/admin/messages` — pulls from `contactMessages`.
  - `/admin` overview pulls recent of both.

## 8. SEO — Google May 2026 alignment

- `<Seo>` is the single per-page meta surface (title template, description,
  canonical, robots, Open Graph, Twitter, hreflang, application-name).
- JSON-LD via `lib/utils/jsonld.ts` + `<JsonLd>`:
  `Organization`, `WebSite` w/ SearchAction (site-wide);
  per-page `Product`, `Course`, `Service`, `Article`, `FAQPage`,
  `BreadcrumbList`, `AboutPage`, `ContactPage`, `Person` (E-E-A-T).
- `/sitemap.xml` and `/robots.txt` are first-class SvelteKit endpoints.
- Performance signals: font-display swap, image dimensions declared, route-level
  prerender for marketing routes is set up (toggleable in `+page.ts`).
- AI-search / 2026 readiness: semantic landmarks, author bylines, FAQPage
  schema, and informative summaries at top of every section.
- WCAG 2.2 AA contrast is validated on every gold-on-dark pairing in tokens.

## 9. Accessibility

- `:focus-visible` outline using `--gold-400`, skip-to-content link in
  `app.html`, `prefers-reduced-motion` honoured globally, keyboard-trappable
  modals + cart drawer, ARIA landmarks throughout.

## 10. NPM scripts

```
dev          vite dev
build        vite build
preview      vite preview
check        svelte-kit sync && svelte-check
lint         prettier --check . && eslint .
format       prettier --write .
db:generate  drizzle-kit generate
db:push      drizzle-kit push
db:studio    drizzle-kit studio
db:seed      tsx src/lib/server/db/seed.ts
```

## 11. Conventions

- **One responsibility per Svelte file.** Anything reusable lives in `lib/components`.
- **No external SVG.** Enforced by ESLint. Icons → Tabler. Logo marks are CSS +
  Tabler. Charts are HTML `<canvas>` drawn programmatically.
- **No `tailwind`, no Bootstrap, no UI library.** Plain CSS with tokens.
- **Snippets over slots.** All component composition uses Svelte 5 snippets.
- **Untrack on derived initial state.** When seeding `$state` from props at
  mount time, wrap with `untrack(() => …)` to avoid `state_referenced_locally`
  warnings.
- **`page.url.pathname` is treated as `string`** in nav helpers — SvelteKit's
  typed route union is a refinement, not a contract.

## 12. Phase 1 explicit limits

- Auth is stubbed (mock user in `event.locals`; login/signup set a cookie role).
- Checkout is intentionally inert (cart, drawer, summary all work — the
  "Continue" button is disabled with an explicit "Phase 2" tooltip).
- No ESP integration; lead-magnet & contact form persist to SQLite only.
- Admin "write" surfaces (product CRUD, plans, etc.) are UI-complete but stub
  on submit. Read paths (`leads`, `messages`) are live.
