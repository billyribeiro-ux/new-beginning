import { browser } from '$app/environment';

const SIDEBAR_KEY = 'tradeflex:sidebar:collapsed:v1';

class UiStore {
	mobileNavOpen = $state(false);
	sidebarCollapsed = $state(false);
	commandPaletteOpen = $state(false);

	constructor() {
		if (browser) {
			try {
				this.sidebarCollapsed = localStorage.getItem(SIDEBAR_KEY) === '1';
			} catch {
				/* ignore */
			}
			$effect.root(() => {
				$effect(() => {
					try {
						localStorage.setItem(SIDEBAR_KEY, this.sidebarCollapsed ? '1' : '0');
					} catch {
						/* ignore */
					}
				});
			});
		}
	}

	toggleSidebar() {
		this.sidebarCollapsed = !this.sidebarCollapsed;
	}

	openMobileNav() {
		this.mobileNavOpen = true;
	}
	closeMobileNav() {
		this.mobileNavOpen = false;
	}
}

export const ui = new UiStore();
