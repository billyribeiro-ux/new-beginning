import { browser } from '$app/environment';
import { gsap } from 'gsap';
import { ScrollTrigger } from 'gsap/ScrollTrigger';

let registered = false;

export function getGsap(): typeof gsap | null {
	if (!browser) return null;
	if (!registered) {
		gsap.registerPlugin(ScrollTrigger);
		registered = true;
		gsap.defaults({ ease: 'expo.out', duration: 1.1 });
	}
	return gsap;
}

export function getScrollTrigger() {
	if (!browser) return null;
	getGsap();
	return ScrollTrigger;
}

export function prefersReducedMotion(): boolean {
	if (!browser) return false;
	return window.matchMedia('(prefers-reduced-motion: reduce)').matches;
}
