import { isTauri } from '$lib/platform';

type InvokeFn = (cmd: string, args?: Record<string, unknown>) => Promise<unknown>;

let _invoke: InvokeFn | null = null;

async function getInvoke(): Promise<InvokeFn> {
	if (_invoke) return _invoke;
	const { invoke } = await import('@tauri-apps/api/core');
	_invoke = invoke;
	return _invoke;
}

export async function tauriInvoke<T>(
	cmd: string,
	args?: Record<string, unknown>
): Promise<T> {
	if (!isTauri) {
		throw new Error('tauriInvoke called outside of Tauri context');
	}
	const invoke = await getInvoke();
	return invoke(cmd, args) as Promise<T>;
}
