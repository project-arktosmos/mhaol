import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { get } from 'svelte/store';

describe('ToastService', () => {
	let toastService: (typeof import('../../src/services/toast.service'))['toastService'];

	beforeEach(async () => {
		vi.resetModules();
		vi.useFakeTimers();
		const mod = await import('../../src/services/toast.service');
		toastService = mod.toastService;
	});

	afterEach(() => {
		vi.useRealTimers();
	});

	// ===== Initial state =====

	it('should start with empty toast list', () => {
		const toasts = get(toastService);
		expect(toasts).toEqual([]);
	});

	// ===== add =====

	it('should add a toast with default level and duration', () => {
		toastService.add('Hello');

		const toasts = get(toastService);
		expect(toasts).toHaveLength(1);
		expect(toasts[0].message).toBe('Hello');
		expect(toasts[0].level).toBe('info');
		expect(toasts[0].duration).toBe(4000);
		expect(toasts[0].id).toBeTruthy();
	});

	it('should add a toast with custom level', () => {
		toastService.add('Error occurred', 'error');

		const toasts = get(toastService);
		expect(toasts).toHaveLength(1);
		expect(toasts[0].level).toBe('error');
	});

	it('should add multiple toasts', () => {
		toastService.add('First');
		toastService.add('Second');
		toastService.add('Third');

		const toasts = get(toastService);
		expect(toasts).toHaveLength(3);
		expect(toasts[0].message).toBe('First');
		expect(toasts[1].message).toBe('Second');
		expect(toasts[2].message).toBe('Third');
	});

	// ===== remove =====

	it('should remove a toast by id', () => {
		toastService.add('To remove');
		const toasts = get(toastService);
		const id = toasts[0].id;

		toastService.remove(id);

		expect(get(toastService)).toHaveLength(0);
	});

	it('should only remove the specified toast', () => {
		toastService.add('Keep');
		toastService.add('Remove');

		const toasts = get(toastService);
		const removeId = toasts[1].id;

		toastService.remove(removeId);

		const remaining = get(toastService);
		expect(remaining).toHaveLength(1);
		expect(remaining[0].message).toBe('Keep');
	});

	it('should handle removing a non-existent id gracefully', () => {
		toastService.add('Stays');
		toastService.remove('non-existent-id');

		expect(get(toastService)).toHaveLength(1);
	});

	// ===== Auto-dismiss =====

	it('should auto-dismiss after default duration', () => {
		toastService.add('Auto dismiss');

		expect(get(toastService)).toHaveLength(1);

		vi.advanceTimersByTime(4000);

		expect(get(toastService)).toHaveLength(0);
	});

	it('should auto-dismiss after custom duration', () => {
		toastService.add('Quick', 'info', 1000);

		expect(get(toastService)).toHaveLength(1);

		vi.advanceTimersByTime(999);
		expect(get(toastService)).toHaveLength(1);

		vi.advanceTimersByTime(1);
		expect(get(toastService)).toHaveLength(0);
	});

	it('should not auto-dismiss when duration is 0', () => {
		toastService.add('Persistent', 'info', 0);

		vi.advanceTimersByTime(10000);

		expect(get(toastService)).toHaveLength(1);
	});

	// ===== Convenience methods =====

	it('should add info toast via info()', () => {
		toastService.info('Info message');

		const toasts = get(toastService);
		expect(toasts[0].level).toBe('info');
		expect(toasts[0].message).toBe('Info message');
	});

	it('should add success toast via success()', () => {
		toastService.success('Success message');

		const toasts = get(toastService);
		expect(toasts[0].level).toBe('success');
	});

	it('should add warning toast via warning()', () => {
		toastService.warning('Warning message');

		const toasts = get(toastService);
		expect(toasts[0].level).toBe('warning');
	});

	it('should add error toast via error()', () => {
		toastService.error('Error message');

		const toasts = get(toastService);
		expect(toasts[0].level).toBe('error');
	});

	it('should respect custom duration on convenience methods', () => {
		toastService.info('Quick info', 500);

		vi.advanceTimersByTime(500);

		expect(get(toastService)).toHaveLength(0);
	});

	// ===== addWithActions =====

	it('should add a toast with actions', () => {
		const action = { label: 'Undo', onclick: vi.fn() };
		const id = toastService.addWithActions('Deleted item', [action]);

		const toasts = get(toastService);
		expect(toasts).toHaveLength(1);
		expect(toasts[0].actions).toHaveLength(1);
		expect(toasts[0].actions![0].label).toBe('Undo');
		expect(id).toBeTruthy();
	});

	it('should not auto-dismiss action toast when duration is 0 (default)', () => {
		toastService.addWithActions('Action toast', [{ label: 'OK', onclick: vi.fn() }]);

		vi.advanceTimersByTime(10000);

		expect(get(toastService)).toHaveLength(1);
	});

	it('should auto-dismiss action toast when duration is set', () => {
		toastService.addWithActions('Timed action', [{ label: 'OK', onclick: vi.fn() }], 'info', 2000);

		vi.advanceTimersByTime(2000);

		expect(get(toastService)).toHaveLength(0);
	});

	it('should return the toast id from addWithActions', () => {
		const id = toastService.addWithActions('Test', []);

		const toasts = get(toastService);
		expect(toasts[0].id).toBe(id);
	});

	// ===== Unique ids =====

	it('should generate unique ids for each toast', () => {
		toastService.add('A', 'info', 0);
		toastService.add('B', 'info', 0);
		toastService.add('C', 'info', 0);

		const toasts = get(toastService);
		const ids = toasts.map((t) => t.id);
		const uniqueIds = new Set(ids);
		expect(uniqueIds.size).toBe(3);
	});
});
