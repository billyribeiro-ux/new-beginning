import { browser } from '$app/environment';

class MotionStore {
	reduced = $state(false);

	constructor() {
		if (!browser) return;
		const mq = window.matchMedia('(prefers-reduced-motion: reduce)');
		this.reduced = mq.matches;
		const handler = (e: MediaQueryListEvent) => {
			this.reduced = e.matches;
		};
		mq.addEventListener('change', handler);
	}
}

export const motion = new MotionStore();
