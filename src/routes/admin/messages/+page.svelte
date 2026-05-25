<script lang="ts">
	import type { PageData } from './$types';
	import { IconSearch, IconMail, IconMessages } from '@tabler/icons-svelte';
	import Seo from '$lib/components/seo/Seo.svelte';
	import EmptyState from '$lib/components/dashboard/EmptyState.svelte';
	import Badge from '$lib/components/ui/Badge.svelte';

	let { data }: { data: PageData } = $props();
	let query = $state('');
	let activeId = $state<string | null>(null);

	const filtered = $derived(
		data.messages.filter(
			(m) =>
				!query.trim() ||
				m.name.toLowerCase().includes(query.toLowerCase()) ||
				m.subject.toLowerCase().includes(query.toLowerCase()) ||
				m.email.toLowerCase().includes(query.toLowerCase())
		)
	);
	const active = $derived(filtered.find((m) => m.id === activeId) ?? filtered[0]);

	function fmtDate(ts: Date | number) {
		const d = ts instanceof Date ? ts : new Date(ts);
		return d.toLocaleString('en-US', {
			month: 'short',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		});
	}
</script>

<Seo title="Admin · Messages" noindex />

<header class="ph">
	<div>
		<p class="eyebrow">Audience</p>
		<h2>Contact inbox</h2>
		<p class="muted">{data.messages.length} messages from the public contact form.</p>
	</div>
</header>

{#if data.messages.length === 0}
	<EmptyState
		icon={IconMessages}
		title="No messages yet"
		description="Submit the public contact form to see the first message land here."
	/>
{:else}
	<div class="inbox">
		<aside class="list">
			<div class="search">
				<IconSearch size={14} />
				<input
					type="search"
					placeholder="Search…"
					bind:value={query}
					aria-label="Search messages"
				/>
			</div>
			<ul>
				{#each filtered as m (m.id)}
					<li>
						<button
							type="button"
							class="m-row"
							class:is-active={active?.id === m.id}
							onclick={() => (activeId = m.id)}
						>
							<div class="m-head">
								<strong>{m.name}</strong>
								<span class="m-when">{fmtDate(m.createdAt)}</span>
							</div>
							<p class="m-sub">{m.subject}</p>
							<p class="m-prev">{m.body.slice(0, 80)}{m.body.length > 80 ? '…' : ''}</p>
						</button>
					</li>
				{/each}
			</ul>
		</aside>
		<article class="reader">
			{#if active}
				<header class="r-head">
					<div>
						<h3>{active.subject}</h3>
						<p class="from">
							<span class="avatar">{active.name[0]?.toUpperCase()}</span>{active.name} ·
							<a href="mailto:{active.email}">{active.email}</a>
						</p>
						<p class="when">{fmtDate(active.createdAt)}</p>
					</div>
					<a class="reply" href="mailto:{active.email}?subject=Re: {active.subject}">
						<IconMail size={14} />
						Reply via email
					</a>
				</header>
				<div class="body">
					{#each active.body.split('\n').filter(Boolean) as para, i (i)}
						<p>{para}</p>
					{/each}
				</div>
				<footer class="meta-row">
					<Badge variant="outline" size="sm">Contact form</Badge>
					<span class="id">ID · {active.id}</span>
				</footer>
			{/if}
		</article>
	</div>
{/if}

<style>
	.ph {
		margin-bottom: clamp(2rem, 4vw, 3rem);
	}
	.eyebrow {
		font-size: var(--text-2xs);
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		color: var(--gold-400);
		font-weight: var(--weight-semibold);
		margin: 0;
	}
	.eyebrow::before {
		display: none;
	}
	.ph h2 {
		font-family: var(--font-display);
		font-size: clamp(1.75rem, 3vw, 2.5rem);
		margin: var(--space-2) 0;
	}
	.muted {
		color: var(--ink-400);
		font-size: var(--text-sm);
		margin: 0;
	}

	.inbox {
		display: grid;
		grid-template-columns: 1fr;
		gap: var(--space-4);
		min-height: 600px;
	}
	@media (--bp-lg) {
		.inbox {
			grid-template-columns: 360px 1fr;
		}
	}

	.list {
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
		background: var(--surface-1);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-lg);
		padding: var(--space-3);
		max-height: 80vh;
		overflow-y: auto;
	}
	.search {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		padding: var(--space-2) var(--space-3);
		background: var(--surface-2);
		border-radius: var(--radius-full);
		color: var(--ink-400);
	}
	.search input {
		flex: 1;
		background: transparent;
		border: 0;
		color: var(--ink-100);
		font-size: var(--text-sm);
	}
	.search input:focus {
		outline: none;
	}

	.list ul {
		list-style: none;
		display: flex;
		flex-direction: column;
		gap: var(--space-1);
	}
	.m-row {
		width: 100%;
		text-align: left;
		padding: var(--space-3);
		background: transparent;
		border: 0;
		border-radius: var(--radius-md);
		cursor: pointer;
		display: flex;
		flex-direction: column;
		gap: 4px;
		transition: background var(--dur-fast) var(--ease-out);
	}
	.m-row:hover {
		background: var(--surface-2);
	}
	.m-row.is-active {
		background: var(--surface-3);
		box-shadow: inset 3px 0 0 var(--gold-400);
	}
	.m-head {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}
	.m-head strong {
		font-size: var(--text-sm);
		color: var(--ink-100);
	}
	.m-when {
		font-size: 10px;
		color: var(--ink-400);
	}
	.m-sub {
		font-size: var(--text-xs);
		color: var(--ink-200);
		margin: 0;
		font-weight: var(--weight-medium);
	}
	.m-prev {
		font-size: var(--text-xs);
		color: var(--ink-400);
		margin: 0;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.reader {
		padding: clamp(1.5rem, 3vw, 2.5rem);
		background: var(--surface-1);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-lg);
		display: flex;
		flex-direction: column;
		gap: var(--space-5);
	}
	.r-head {
		display: flex;
		justify-content: space-between;
		gap: var(--space-3);
		flex-wrap: wrap;
		padding-bottom: var(--space-4);
		border-bottom: 1px solid var(--border-default);
	}
	.r-head h3 {
		font-family: var(--font-display);
		font-size: var(--text-2xl);
		margin: 0 0 var(--space-3);
	}
	.from {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		font-size: var(--text-sm);
		color: var(--ink-200);
		margin: 0;
	}
	.avatar {
		width: 28px;
		height: 28px;
		background: var(--gradient-gold);
		color: var(--surface-0);
		display: inline-flex;
		align-items: center;
		justify-content: center;
		border-radius: var(--radius-full);
		font-weight: var(--weight-bold);
		font-size: 10px;
		margin-inline-end: var(--space-2);
	}
	.from a {
		color: var(--gold-300);
	}
	.when {
		font-size: var(--text-xs);
		color: var(--ink-400);
		margin: var(--space-2) 0 0;
	}
	.reply {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: var(--space-3) var(--space-4);
		background: var(--gradient-gold);
		color: var(--surface-0);
		font-size: var(--text-sm);
		font-weight: var(--weight-semibold);
		border-radius: var(--radius-full);
		text-decoration: none;
		height: max-content;
	}
	.body {
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
		color: var(--ink-200);
		font-size: var(--text-md);
		line-height: var(--leading-relaxed);
	}
	.meta-row {
		display: flex;
		justify-content: space-between;
		padding-top: var(--space-4);
		border-top: 1px solid var(--border-default);
		font-size: var(--text-xs);
	}
	.id {
		font-family: var(--font-mono);
		color: var(--ink-400);
	}
</style>
