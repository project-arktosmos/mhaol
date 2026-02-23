import { writable } from 'svelte/store';

const isBrowser = typeof window !== 'undefined' && typeof document !== 'undefined';

export default function localStorageWritableStore<T>(key: string, initialValue: T) {
	if (!isBrowser) {
		return writable(initialValue);
	}
	const storedValue = localStorage.getItem(key);
	const parsedValue = storedValue ? JSON.parse(storedValue) : initialValue;
	const store = writable<T>(parsedValue);
	store.subscribe((value) => {
		localStorage.setItem(key, JSON.stringify(value));
	});
	return store;
}