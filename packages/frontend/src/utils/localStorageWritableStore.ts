import { writable } from 'svelte/store';

const isBrowser = typeof window !== 'undefined' && typeof document !== 'undefined';

export default function localStorageWritableStore<T>(key: string, initialValue: T) {
	if (!isBrowser) {
		return writable(initialValue);
	}
	const storedValue = localStorage.getItem(key);
	// Merge stored value with initial defaults so newly added fields are always present
	const parsedValue = storedValue
		? typeof initialValue === 'object' && initialValue !== null && !Array.isArray(initialValue)
			? { ...initialValue, ...JSON.parse(storedValue) }
			: JSON.parse(storedValue)
		: initialValue;
	const store = writable<T>(parsedValue);
	store.subscribe((value) => {
		try {
			localStorage.setItem(key, JSON.stringify(value));
		} catch {
			// Quota exceeded or storage disabled — drop this entry and keep going
			// rather than letting an unhandled rejection escape into app boot.
			try {
				localStorage.removeItem(key);
			} catch {
				// ignore — nothing more we can do
			}
		}
	});
	return store;
}
