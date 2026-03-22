export type ToastLevel = 'info' | 'success' | 'warning' | 'error';

export interface ToastAction {
	label: string;
	onclick: () => void;
}

export interface Toast {
	id: string;
	message: string;
	level: ToastLevel;
	duration: number;
	actions?: ToastAction[];
}
