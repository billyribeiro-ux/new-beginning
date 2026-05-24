/**
 * Svelte 5 attachments wrapping GSAP timelines.
 * Pattern: each attachment returns a (node) => cleanup function.
 * GSAP context auto-cleans on unmount via ctx.revert().
 */
import { getGsap, getScrollTrigger, prefersReducedMotion } from './gsap.js';
import type { Attachment } from 'svelte/attachments';

interface RevealOpts {
	y?: number;
	x?: number;
	scale?: number;
	delay?: number;
	duration?: number;
	stagger?: number;
	start?: string;
	once?: boolean;
}

/** Fade-up reveal on scroll entrance. */
export function fadeUp(opts: RevealOpts = {}): Attachment<HTMLElement> {
	return (node) => {
		const gsap = getGsap();
		const ST = getScrollTrigger();
		if (!gsap || !ST) {
			node.style.opacity = '1';
			node.style.transform = 'none';
			return;
		}
		if (prefersReducedMotion()) {
			gsap.set(node, { opacity: 1, y: 0, x: 0, scale: 1 });
			return;
		}
		const ctx = gsap.context(() => {
			gsap.fromTo(
				node,
				{ opacity: 0, y: opts.y ?? 28, x: opts.x ?? 0, scale: opts.scale ?? 1 },
				{
					opacity: 1,
					y: 0,
					x: 0,
					scale: 1,
					duration: opts.duration ?? 1.1,
					delay: opts.delay ?? 0,
					ease: 'expo.out',
					scrollTrigger: {
						trigger: node,
						start: opts.start ?? 'top 85%',
						toggleActions: opts.once === false ? 'play none none reverse' : 'play none none none'
					}
				}
			);
		}, node);
		return () => ctx.revert();
	};
}

/** Staggered children reveal — applies to direct children. */
export function stagger(
	opts: RevealOpts & { childSelector?: string } = {}
): Attachment<HTMLElement> {
	return (node) => {
		const gsap = getGsap();
		const ST = getScrollTrigger();
		if (!gsap || !ST) return;
		const children = node.querySelectorAll<HTMLElement>(opts.childSelector ?? ':scope > *');
		if (!children.length) return;
		if (prefersReducedMotion()) {
			gsap.set(children, { opacity: 1, y: 0 });
			return;
		}
		const ctx = gsap.context(() => {
			gsap.fromTo(
				children,
				{ opacity: 0, y: opts.y ?? 32 },
				{
					opacity: 1,
					y: 0,
					duration: opts.duration ?? 0.9,
					ease: 'expo.out',
					stagger: opts.stagger ?? 0.08,
					scrollTrigger: { trigger: node, start: opts.start ?? 'top 80%' }
				}
			);
		}, node);
		return () => ctx.revert();
	};
}

/** Split a string into <span> per word for character-level animation. */
function splitToWords(node: HTMLElement) {
	const text = node.textContent ?? '';
	const words = text.split(/(\s+)/);
	node.textContent = '';
	const spans: HTMLSpanElement[] = [];
	for (const w of words) {
		if (/^\s+$/.test(w)) {
			node.appendChild(document.createTextNode(w));
			continue;
		}
		const wrap = document.createElement('span');
		wrap.className = 'split-word';
		wrap.style.display = 'inline-block';
		wrap.style.overflow = 'hidden';
		wrap.style.verticalAlign = 'top';
		const inner = document.createElement('span');
		inner.className = 'split-word-inner';
		inner.style.display = 'inline-block';
		inner.style.willChange = 'transform, opacity';
		inner.textContent = w;
		wrap.appendChild(inner);
		node.appendChild(wrap);
		spans.push(inner);
	}
	return spans;
}

/** Headline split-reveal — used on hero h1. */
export function splitReveal(
	opts: { stagger?: number; delay?: number } = {}
): Attachment<HTMLElement> {
	return (node) => {
		const gsap = getGsap();
		if (!gsap) return;
		if (prefersReducedMotion()) return;
		const original = node.innerHTML;
		const spans = splitToWords(node);
		const ctx = gsap.context(() => {
			gsap.set(spans, { y: '110%', opacity: 0 });
			gsap.to(spans, {
				y: '0%',
				opacity: 1,
				duration: 1.2,
				ease: 'expo.out',
				stagger: opts.stagger ?? 0.06,
				delay: opts.delay ?? 0.15
			});
		}, node);
		return () => {
			ctx.revert();
			node.innerHTML = original;
		};
	};
}

/** Magnetic hover effect on premium CTAs and pricing cards. */
export function magnetic(opts: { strength?: number } = {}): Attachment<HTMLElement> {
	return (node) => {
		if (prefersReducedMotion()) return;
		const gsap = getGsap();
		if (!gsap) return;
		const strength = opts.strength ?? 0.35;
		let raf = 0;
		const onMove = (e: PointerEvent) => {
			const rect = node.getBoundingClientRect();
			const cx = rect.left + rect.width / 2;
			const cy = rect.top + rect.height / 2;
			const dx = (e.clientX - cx) * strength;
			const dy = (e.clientY - cy) * strength;
			cancelAnimationFrame(raf);
			raf = requestAnimationFrame(() => {
				gsap.to(node, { x: dx, y: dy, duration: 0.6, ease: 'expo.out' });
			});
		};
		const onLeave = () => {
			gsap.to(node, { x: 0, y: 0, duration: 0.8, ease: 'elastic.out(1, 0.4)' });
		};
		node.addEventListener('pointermove', onMove);
		node.addEventListener('pointerleave', onLeave);
		return () => {
			node.removeEventListener('pointermove', onMove);
			node.removeEventListener('pointerleave', onLeave);
			gsap.set(node, { x: 0, y: 0 });
		};
	};
}

/** Animate a number from 0 → target as it scrolls into view. */
export function numberTicker(
	opts: {
		to: number;
		from?: number;
		duration?: number;
		format?: (n: number) => string;
		start?: string;
	} = { to: 0 }
): Attachment<HTMLElement> {
	return (node) => {
		const gsap = getGsap();
		const ST = getScrollTrigger();
		if (!gsap || !ST) {
			node.textContent = (opts.format ?? String)(opts.to);
			return;
		}
		if (prefersReducedMotion()) {
			node.textContent = (opts.format ?? String)(opts.to);
			return;
		}
		const fmt = opts.format ?? ((n: number) => Math.round(n).toLocaleString());
		const proxy = { v: opts.from ?? 0 };
		node.textContent = fmt(proxy.v);
		const ctx = gsap.context(() => {
			gsap.to(proxy, {
				v: opts.to,
				duration: opts.duration ?? 2.2,
				ease: 'expo.out',
				onUpdate: () => {
					node.textContent = fmt(proxy.v);
				},
				scrollTrigger: { trigger: node, start: opts.start ?? 'top 90%' }
			});
		}, node);
		return () => ctx.revert();
	};
}

/** Subtle parallax on background layers. */
export function parallax(opts: { speed?: number } = {}): Attachment<HTMLElement> {
	return (node) => {
		const gsap = getGsap();
		const ST = getScrollTrigger();
		if (!gsap || !ST) return;
		if (prefersReducedMotion()) return;
		const speed = opts.speed ?? 0.25;
		const ctx = gsap.context(() => {
			gsap.to(node, {
				yPercent: -100 * speed,
				ease: 'none',
				scrollTrigger: {
					trigger: node.parentElement ?? node,
					start: 'top bottom',
					end: 'bottom top',
					scrub: 0.6
				}
			});
		}, node);
		return () => ctx.revert();
	};
}

/** Cursor-follow gold glow on premium sections. */
export function cursorGlow(): Attachment<HTMLElement> {
	return (node) => {
		if (prefersReducedMotion()) return;
		if (matchMedia('(pointer: coarse)').matches) return;
		const onMove = (e: PointerEvent) => {
			const rect = node.getBoundingClientRect();
			node.style.setProperty('--glow-x', `${e.clientX - rect.left}px`);
			node.style.setProperty('--glow-y', `${e.clientY - rect.top}px`);
		};
		node.addEventListener('pointermove', onMove);
		return () => node.removeEventListener('pointermove', onMove);
	};
}
