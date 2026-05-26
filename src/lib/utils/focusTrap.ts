import type { Attachment } from 'svelte/attachments';

const FOCUSABLE_SELECTOR =
	'a[href], button:not([disabled]), textarea:not([disabled]), input:not([disabled]), select:not([disabled]), [tabindex]:not([tabindex="-1"])';

/**
 * Svelte 5 attachment that traps keyboard focus inside the host element.
 *
 * - Saves the previously focused element on mount and restores it on destroy.
 * - Moves initial focus to the first focusable child (or the host itself).
 * - Intercepts Tab / Shift+Tab to cycle within the host's focusables.
 *
 * Re-queries focusables on each keydown so dynamically added/removed
 * children are accounted for.
 */
export const focusTrap: Attachment<HTMLElement> = (node) => {
	const previouslyFocused = document.activeElement as HTMLElement | null;

	const getFocusable = (): HTMLElement[] =>
		Array.from(node.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTOR));

	// Initial focus
	let tempTabindexApplied = false;
	const initial = getFocusable();
	const firstInitial = initial[0];
	if (firstInitial) {
		firstInitial.focus({ preventScroll: true });
	} else {
		if (!node.hasAttribute('tabindex')) {
			node.setAttribute('tabindex', '-1');
			tempTabindexApplied = true;
		}
		node.focus({ preventScroll: true });
	}

	function onKeydown(e: KeyboardEvent) {
		if (e.key !== 'Tab') return;

		const focusable = getFocusable();
		const first = focusable[0];
		const last = focusable[focusable.length - 1];
		if (!first || !last) {
			e.preventDefault();
			node.focus({ preventScroll: true });
			return;
		}

		const active = document.activeElement as HTMLElement | null;

		if (e.shiftKey) {
			if (active === first || !node.contains(active)) {
				e.preventDefault();
				last.focus({ preventScroll: true });
			}
		} else {
			if (active === last || !node.contains(active)) {
				e.preventDefault();
				first.focus({ preventScroll: true });
			}
		}
	}

	node.addEventListener('keydown', onKeydown);

	return () => {
		node.removeEventListener('keydown', onKeydown);
		if (tempTabindexApplied) {
			node.removeAttribute('tabindex');
		}
		if (previouslyFocused && document.contains(previouslyFocused)) {
			previouslyFocused.focus({ preventScroll: true });
		}
	};
};
