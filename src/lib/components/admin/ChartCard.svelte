<script lang="ts">
	import { browser } from '$app/environment';
	import { prefersReducedMotion } from '$lib/animations/gsap.js';

	import type { Snippet } from 'svelte';

	type Props = {
		title: string;
		subtitle?: string;
		data: number[];
		labels?: string[];
		height?: number;
		color?: string;
		actions?: Snippet;
	};
	let { title, subtitle, data, labels, height = 220, color = '#E8B660', actions }: Props = $props();

	let canvasEl: HTMLCanvasElement | undefined = $state();

	$effect(() => {
		if (!browser || !canvasEl) return;
		const ctx = canvasEl.getContext('2d');
		if (!ctx) return;
		const dpr = window.devicePixelRatio || 1;
		const w = canvasEl.clientWidth;
		const h = canvasEl.clientHeight;
		canvasEl.width = Math.floor(w * dpr);
		canvasEl.height = Math.floor(h * dpr);
		ctx.scale(dpr, dpr);

		ctx.clearRect(0, 0, w, h);
		if (data.length < 2) return;

		const max = Math.max(...data);
		const min = Math.min(...data);
		const range = max - min || 1;
		const padL = 36;
		const padR = 12;
		const padT = 16;
		const padB = 28;
		const innerW = w - padL - padR;
		const innerH = h - padT - padB;

		// gridlines
		ctx.strokeStyle = 'rgba(255,255,255,0.05)';
		ctx.lineWidth = 1;
		ctx.font = '10px "JetBrains Mono", monospace';
		ctx.fillStyle = 'rgba(160,158,143,0.6)';
		for (let i = 0; i <= 4; i++) {
			const y = padT + (innerH / 4) * i;
			ctx.beginPath();
			ctx.moveTo(padL, y);
			ctx.lineTo(padL + innerW, y);
			ctx.stroke();
			const v = max - (range / 4) * i;
			ctx.fillText(formatTick(v), 4, y + 3);
		}

		const points = data.map((v, i) => ({
			x: padL + (i / (data.length - 1)) * innerW,
			y: padT + innerH - ((v - min) / range) * innerH
		}));

		// area fill
		const grad = ctx.createLinearGradient(0, padT, 0, padT + innerH);
		grad.addColorStop(0, 'rgba(232, 182, 96, 0.32)');
		grad.addColorStop(1, 'rgba(232, 182, 96, 0)');
		ctx.fillStyle = grad;
		ctx.beginPath();
		ctx.moveTo(points[0]!.x, padT + innerH);
		for (const p of points) ctx.lineTo(p.x, p.y);
		ctx.lineTo(points[points.length - 1]!.x, padT + innerH);
		ctx.closePath();
		ctx.fill();

		// line
		ctx.strokeStyle = color;
		ctx.lineWidth = 2;
		ctx.lineJoin = 'round';
		ctx.lineCap = 'round';
		ctx.beginPath();
		for (let i = 0; i < points.length; i++) {
			const p = points[i]!;
			if (i === 0) ctx.moveTo(p.x, p.y);
			else ctx.lineTo(p.x, p.y);
		}
		ctx.stroke();

		// dots at last point
		const last = points[points.length - 1]!;
		ctx.fillStyle = color;
		ctx.beginPath();
		ctx.arc(last.x, last.y, 4, 0, Math.PI * 2);
		ctx.fill();
		ctx.fillStyle = 'rgba(232, 182, 96, 0.18)';
		ctx.beginPath();
		ctx.arc(last.x, last.y, 9, 0, Math.PI * 2);
		ctx.fill();

		// x labels
		if (labels?.length) {
			ctx.fillStyle = 'rgba(160,158,143,0.6)';
			ctx.textAlign = 'center';
			const step = Math.max(1, Math.ceil(labels.length / 6));
			for (let i = 0; i < labels.length; i += step) {
				const x = padL + (i / (labels.length - 1)) * innerW;
				ctx.fillText(labels[i]!, x, h - 8);
			}
			ctx.textAlign = 'start';
		}

		if (prefersReducedMotion()) return;
	});

	function formatTick(v: number) {
		if (v >= 1_000_000) return `${(v / 1_000_000).toFixed(1)}M`;
		if (v >= 1_000) return `${(v / 1_000).toFixed(0)}k`;
		return v.toFixed(0);
	}
</script>

<article class="chart-card">
	<header>
		<div>
			<h3>{title}</h3>
			{#if subtitle}<p>{subtitle}</p>{/if}
		</div>
		{#if actions}{@render actions()}{/if}
	</header>
	<div class="canvas-wrap" style:height="{height}px">
		<canvas bind:this={canvasEl} aria-label="{title} chart"></canvas>
	</div>
</article>

<style>
	.chart-card {
		padding: var(--space-6);
		background: var(--surface-1);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-lg);
	}
	header {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		gap: var(--space-3);
		margin-bottom: var(--space-4);
	}
	h3 {
		font-family: var(--font-display);
		font-size: var(--text-lg);
		margin: 0;
	}
	header p {
		font-size: var(--text-xs);
		color: var(--ink-400);
		margin: 4px 0 0;
	}
	.canvas-wrap {
		width: 100%;
	}
	canvas {
		width: 100%;
		height: 100%;
		display: block;
	}
</style>
