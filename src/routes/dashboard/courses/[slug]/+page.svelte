<script lang="ts">
	import type { PageData } from './$types';
	import {
		IconPlayerPlay,
		IconCheck,
		IconLock,
		IconChevronLeft,
		IconChevronRight,
		IconBookmark,
		IconNotes,
		IconFileText,
		IconDownload
	} from '@tabler/icons-svelte';
	import Seo from '$lib/components/seo/Seo.svelte';
	import Tabs from '$lib/components/ui/Tabs.svelte';
	import ProgressBar from '$lib/components/dashboard/ProgressBar.svelte';
	import Button from '$lib/components/ui/Button.svelte';

	let { data }: { data: PageData } = $props();
	const c = $derived(data.course);

	const modules = [
		{
			id: 1,
			title: 'Options Mechanics',
			lessons: [
				{ id: '1.1', title: 'Why options exist', dur: '8:12', done: true },
				{ id: '1.2', title: 'Calls, puts, and contract anatomy', dur: '11:48', done: true },
				{ id: '1.3', title: 'Settlement and assignment', dur: '14:02', done: true },
				{ id: '1.4', title: 'Mini-quiz', dur: '5:00', done: true }
			]
		},
		{
			id: 2,
			title: 'Pricing & The Black-Scholes Intuition',
			lessons: [
				{ id: '2.1', title: 'The pricing problem', dur: '12:30', done: true },
				{ id: '2.2', title: 'Intrinsic vs extrinsic', dur: '9:45', done: true },
				{ id: '2.3', title: 'IV in plain language', dur: '14:20', done: false, current: true },
				{ id: '2.4', title: 'Module check-in', dur: '5:00', done: false }
			]
		},
		{
			id: 3,
			title: 'Delta',
			lessons: [
				{ id: '3.1', title: 'Delta as exposure', dur: '10:00', done: false, locked: false },
				{ id: '3.2', title: 'Delta as probability', dur: '13:00', done: false, locked: true },
				{ id: '3.3', title: 'Delta hedging primer', dur: '15:00', done: false, locked: true },
				{ id: '3.4', title: 'Workshop', dur: '20:00', done: false, locked: true }
			]
		}
	];

	let activeTab = $state('overview');
	const tabs = [
		{ id: 'overview', label: 'Overview' },
		{ id: 'notes', label: 'Notes' },
		{ id: 'transcript', label: 'Transcript' },
		{ id: 'resources', label: 'Resources' }
	];

	const allLessons = modules.flatMap((m) => m.lessons);
	const completed = allLessons.filter((l) => l.done).length;
	const progress = Math.round((completed / allLessons.length) * 100);
	const current = allLessons.find((l) => (l as { current?: boolean }).current);
</script>

<Seo title={c.name} noindex />

<div class="layout">
	<aside class="sidebar">
		<a class="back" href="/dashboard/courses"><IconChevronLeft size={14} />Back to courses</a>
		<header class="ch">
			<h2>{c.name}</h2>
			<p class="muted">{c.tagline}</p>
			<ProgressBar value={progress} label="Course progress" />
		</header>
		<nav class="modules">
			{#each modules as m}
				<div class="module">
					<h3>Module {m.id} · {m.title}</h3>
					<ul>
						{#each m.lessons as l}
							{@const locked = 'locked' in l && l.locked}
							{@const isCurrent = 'current' in l && l.current}
							<li class:done={l.done} class:current={isCurrent} class:locked>
								<button type="button" class="lesson" disabled={locked}>
									<span class="ic">
										{#if locked}<IconLock size={12} />{:else if l.done}<IconCheck
												size={12}
												stroke={3}
											/>{:else}<IconPlayerPlay size={12} />{/if}
									</span>
									<span class="lt">{l.id} · {l.title}</span>
									<span class="ld">{l.dur}</span>
								</button>
							</li>
						{/each}
					</ul>
				</div>
			{/each}
		</nav>
	</aside>

	<main class="content">
		<div class="video-wrap">
			<div class="video-frame" aria-label="Video player placeholder">
				<div class="video-bg" style:background={c.media.posterColor}></div>
				<button class="play-big" type="button" aria-label="Play lesson">
					<IconPlayerPlay size={36} />
				</button>
				<div class="video-overlay">
					<p class="lesson-tag">Current lesson</p>
					<h3>{current?.title ?? 'Lesson placeholder'}</h3>
				</div>
				<div class="video-controls">
					<button type="button" aria-label="Previous"><IconChevronLeft size={16} /></button>
					<div class="scrubber"><div class="scrub-fill"></div></div>
					<span class="time">14:20</span>
					<button type="button" aria-label="Bookmark"><IconBookmark size={14} /></button>
					<button type="button" aria-label="Next"><IconChevronRight size={16} /></button>
				</div>
			</div>
		</div>

		<div class="lesson-meta">
			<div>
				<p class="eyebrow">Module 2 · Lesson 3</p>
				<h1>{current?.title ?? 'Lesson title'}</h1>
				<p class="muted">14 min · published May 1, 2026</p>
			</div>
			<div class="lesson-actions">
				<Button variant="gold-outline" size="sm">
					{#snippet iconLeft()}<IconBookmark size={14} />{/snippet}
					Bookmark
				</Button>
				<Button variant="primary" size="sm">
					Mark as complete
					{#snippet iconRight()}<IconCheck size={14} stroke={3} />{/snippet}
				</Button>
			</div>
		</div>

		<Tabs {tabs} bind:value={activeTab} />

		<div class="panel">
			{#if activeTab === 'overview'}
				<h3>About this lesson</h3>
				<p>
					Lorem ipsum dolor sit amet, consectetur adipiscing elit. We walk through implied
					volatility (IV) without calculus — what it actually measures, where to read it on your
					platform, and how it shifts when news breaks. Three screen recordings on SPY, AAPL, and
					NVDA.
				</p>
				<h3>You will learn</h3>
				<ul class="bullets">
					<li>Reading the IV ladder on your platform of choice</li>
					<li>How IV expands and contracts around earnings</li>
					<li>The relationship between IV and option prices</li>
					<li>Common pitfalls when comparing IV across strikes</li>
				</ul>
			{:else if activeTab === 'notes'}
				<h3>Your notes</h3>
				<p class="muted">
					Notes are saved automatically. Highlights you make in the transcript appear here too.
				</p>
				<textarea placeholder="Take a note…"></textarea>
			{:else if activeTab === 'transcript'}
				<h3>Transcript</h3>
				<p class="muted">[00:00:00] Welcome back. Today we tackle implied volatility…</p>
				<p>
					[00:00:24] Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor
					incididunt ut labore et dolore magna aliqua…
				</p>
				<p>
					[00:01:48] Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut
					aliquip ex ea commodo consequat…
				</p>
			{:else if activeTab === 'resources'}
				<h3>Lesson resources</h3>
				<ul class="resource-list">
					<li>
						<span class="ric"><IconFileText size={16} /></span>
						<div>
							<p class="rt">Lesson worksheet</p>
							<p class="rs">PDF · 248 KB</p>
						</div>
						<Button variant="gold-outline" size="sm"
							>{#snippet iconLeft()}<IconDownload size={12} />{/snippet}Download</Button
						>
					</li>
					<li>
						<span class="ric"><IconNotes size={16} /></span>
						<div>
							<p class="rt">Cheat sheet · IV ladder</p>
							<p class="rs">PDF · 112 KB</p>
						</div>
						<Button variant="gold-outline" size="sm"
							>{#snippet iconLeft()}<IconDownload size={12} />{/snippet}Download</Button
						>
					</li>
				</ul>
			{/if}
		</div>
	</main>
</div>

<style>
	.layout {
		display: grid;
		grid-template-columns: 1fr;
		gap: var(--space-5);
		margin: calc(-1 * clamp(1.5rem, 3vw, 2.5rem));
	}
	@media (min-width: 1280px) {
		.layout {
			grid-template-columns: 340px 1fr;
		}
	}

	.sidebar {
		padding: var(--space-5);
		border-right: 1px solid var(--border-default);
		background: var(--surface-1);
		display: flex;
		flex-direction: column;
		gap: var(--space-5);
		min-height: 100%;
	}
	.back {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		font-size: var(--text-xs);
		color: var(--ink-400);
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		text-decoration: none;
		font-weight: var(--weight-semibold);
	}
	.back:hover {
		color: var(--gold-300);
	}
	.ch h2 {
		font-family: var(--font-display);
		font-size: var(--text-xl);
		margin: 0 0 var(--space-2);
	}
	.muted {
		color: var(--ink-400);
		font-size: var(--text-xs);
		margin: 0 0 var(--space-4);
	}

	.modules {
		display: flex;
		flex-direction: column;
		gap: var(--space-5);
	}
	.module h3 {
		font-size: var(--text-xs);
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		color: var(--gold-400);
		margin: 0 0 var(--space-2);
		font-weight: var(--weight-semibold);
	}
	.module ul {
		list-style: none;
		display: flex;
		flex-direction: column;
		gap: 2px;
	}
	.lesson {
		width: 100%;
		display: grid;
		grid-template-columns: 24px 1fr auto;
		gap: var(--space-2);
		padding: var(--space-3);
		background: transparent;
		border: 0;
		border-radius: var(--radius-sm);
		text-align: left;
		font-size: var(--text-xs);
		color: var(--ink-300);
		cursor: pointer;
		transition: all var(--dur-fast) var(--ease-out);
	}
	.lesson:hover:not(:disabled) {
		background: var(--surface-2);
		color: var(--ink-100);
	}
	.ic {
		width: 20px;
		height: 20px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		background: var(--surface-2);
		border-radius: var(--radius-full);
		color: var(--ink-400);
	}
	li.done .ic {
		background: var(--gold-500);
		color: var(--surface-0);
	}
	li.current .ic {
		background: var(--gradient-gold);
		color: var(--surface-0);
	}
	li.current .lesson {
		background: rgba(232, 182, 96, 0.08);
		color: var(--gold-200);
		font-weight: var(--weight-semibold);
	}
	li.locked .lesson {
		opacity: 0.5;
		cursor: not-allowed;
	}
	.lt {
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.ld {
		font-family: var(--font-mono);
		font-size: 10px;
		color: var(--ink-500);
	}

	.content {
		padding: var(--space-5);
		display: flex;
		flex-direction: column;
		gap: var(--space-5);
	}

	.video-wrap {
		width: 100%;
	}
	.video-frame {
		position: relative;
		aspect-ratio: 16 / 9;
		border-radius: var(--radius-xl);
		overflow: hidden;
		background: var(--surface-3);
		display: flex;
	}
	.video-bg {
		position: absolute;
		inset: 0;
	}
	.play-big {
		position: absolute;
		inset: 0;
		display: grid;
		place-items: center;
		color: var(--gold-300);
		background: transparent;
	}
	.play-big::after {
		content: '';
		width: 96px;
		height: 96px;
		background: rgba(0, 0, 0, 0.45);
		border-radius: 50%;
		position: absolute;
		z-index: -1;
		border: 1px solid var(--border-gold);
		backdrop-filter: blur(8px);
	}
	.video-overlay {
		position: absolute;
		top: var(--space-5);
		left: var(--space-5);
		background: rgba(0, 0, 0, 0.55);
		backdrop-filter: blur(8px);
		padding: var(--space-3) var(--space-4);
		border-radius: var(--radius-md);
		border: 1px solid rgba(255, 255, 255, 0.08);
	}
	.lesson-tag {
		font-size: 10px;
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		color: var(--gold-300);
		margin: 0;
		font-weight: var(--weight-semibold);
	}
	.video-overlay h3 {
		font-family: var(--font-display);
		font-size: var(--text-md);
		margin: 4px 0 0;
		color: var(--ink-100);
	}
	.video-controls {
		position: absolute;
		bottom: var(--space-4);
		left: var(--space-4);
		right: var(--space-4);
		display: grid;
		grid-template-columns: auto 1fr auto auto auto;
		gap: var(--space-3);
		align-items: center;
		background: rgba(0, 0, 0, 0.55);
		backdrop-filter: blur(8px);
		padding: var(--space-3);
		border-radius: var(--radius-md);
		border: 1px solid rgba(255, 255, 255, 0.08);
	}
	.video-controls button {
		width: 32px;
		height: 32px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		color: var(--ink-200);
	}
	.scrubber {
		height: 4px;
		background: var(--surface-3);
		border-radius: var(--radius-full);
		overflow: hidden;
	}
	.scrub-fill {
		width: 38%;
		height: 100%;
		background: var(--gradient-gold);
		border-radius: var(--radius-full);
	}
	.time {
		font-family: var(--font-mono);
		font-size: var(--text-xs);
		color: var(--ink-300);
	}

	.lesson-meta {
		display: flex;
		justify-content: space-between;
		align-items: flex-end;
		gap: var(--space-4);
		flex-wrap: wrap;
	}
	.lesson-meta h1 {
		font-family: var(--font-display);
		font-size: var(--text-2xl);
		margin: var(--space-2) 0;
	}
	.lesson-meta .eyebrow {
		font-size: 10px;
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		color: var(--gold-400);
		margin: 0;
		font-weight: var(--weight-semibold);
	}
	.lesson-meta .muted {
		color: var(--ink-400);
		font-size: var(--text-xs);
		margin: 0;
	}
	.lesson-actions {
		display: flex;
		gap: var(--space-2);
	}

	.panel {
		padding: var(--space-6);
		background: var(--surface-1);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-lg);
	}
	.panel h3 {
		font-family: var(--font-display);
		font-size: var(--text-lg);
		margin: var(--space-5) 0 var(--space-3);
	}
	.panel h3:first-child {
		margin-top: 0;
	}
	.panel p {
		color: var(--ink-200);
		font-size: var(--text-sm);
		line-height: var(--leading-relaxed);
		margin: 0 0 var(--space-3);
	}
	.bullets {
		padding-left: var(--space-5);
		color: var(--ink-200);
		font-size: var(--text-sm);
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
	}
	.panel textarea {
		width: 100%;
		min-height: 160px;
		padding: var(--space-4);
		background: var(--surface-2);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-md);
		color: var(--ink-100);
		font-family: inherit;
		font-size: var(--text-sm);
		resize: vertical;
	}

	.resource-list {
		list-style: none;
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
	}
	.resource-list li {
		display: grid;
		grid-template-columns: 32px 1fr auto;
		gap: var(--space-3);
		align-items: center;
		padding: var(--space-4);
		background: var(--surface-2);
		border-radius: var(--radius-md);
	}
	.ric {
		width: 32px;
		height: 32px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		background: var(--surface-3);
		color: var(--gold-400);
		border-radius: var(--radius-full);
	}
	.rt {
		font-size: var(--text-sm);
		color: var(--ink-100);
		margin: 0;
		font-weight: var(--weight-medium);
	}
	.rs {
		font-size: var(--text-xs);
		color: var(--ink-400);
		margin: 4px 0 0;
	}
</style>
