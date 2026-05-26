export type ToastVariant = 'default' | 'success' | 'error' | 'info' | 'warning';

export interface Toast {
	id: string;
	title: string;
	description?: string;
	variant: ToastVariant;
	duration: number;
}

class ToastStore {
	items = $state<Toast[]>([]);

	push(input: { title: string; description?: string; variant?: ToastVariant; duration?: number }) {
		const id = `t-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
		const toast: Toast = {
			id,
			title: input.title,
			description: input.description,
			variant: input.variant ?? 'default',
			duration: input.duration ?? 4200
		};
		this.items = [...this.items, toast];
		if (toast.duration > 0) {
			setTimeout(() => this.dismiss(id), toast.duration);
		}
		return id;
	}

	success(title: string, description?: string) {
		return this.push({ title, description, variant: 'success' });
	}
	error(title: string, description?: string) {
		return this.push({ title, description, variant: 'error' });
	}
	info(title: string, description?: string) {
		return this.push({ title, description, variant: 'info' });
	}

	dismiss(id: string) {
		this.items = this.items.filter((t) => t.id !== id);
	}

	clear() {
		this.items = [];
	}
}

export const toasts = new ToastStore();
