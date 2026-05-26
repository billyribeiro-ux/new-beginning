import { MediaQuery } from 'svelte/reactivity';

class MotionStore {
	#mq = new MediaQuery('prefers-reduced-motion: reduce', false);

	get reduced(): boolean {
		return this.#mq.current;
	}
}

export const motion = new MotionStore();
