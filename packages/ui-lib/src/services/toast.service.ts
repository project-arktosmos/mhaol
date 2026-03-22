import { writable } from 'svelte/store';
import type { Toast, ToastAction, ToastLevel } from 'ui-lib/types/toast.type';

const DEFAULT_DURATION = 4000;

function createToastService() {
	const { subscribe, update } = writable<Toast[]>([]);

	function add(message: string, level: ToastLevel = 'info', duration = DEFAULT_DURATION) {
		const id =
			typeof crypto.randomUUID === 'function'
				? crypto.randomUUID()
				: Math.random().toString(36).slice(2) + Date.now().toString(36);

		const toast: Toast = { id, message, level, duration };
		update((toasts) => [...toasts, toast]);

		if (duration > 0) {
			setTimeout(() => remove(id), duration);
		}
	}

	function remove(id: string) {
		update((toasts) => toasts.filter((t) => t.id !== id));
	}

	function addWithActions(
		message: string,
		actions: ToastAction[],
		level: ToastLevel = 'info',
		duration = 0
	): string {
		const id =
			typeof crypto.randomUUID === 'function'
				? crypto.randomUUID()
				: Math.random().toString(36).slice(2) + Date.now().toString(36);

		const toast: Toast = { id, message, level, duration, actions };
		update((toasts) => [...toasts, toast]);

		if (duration > 0) {
			setTimeout(() => remove(id), duration);
		}
		return id;
	}

	return {
		subscribe,
		add,
		addWithActions,
		remove,
		info: (message: string, duration?: number) => add(message, 'info', duration),
		success: (message: string, duration?: number) => add(message, 'success', duration),
		warning: (message: string, duration?: number) => add(message, 'warning', duration),
		error: (message: string, duration?: number) => add(message, 'error', duration)
	};
}

export const toastService = createToastService();
