import { waitLocale } from 'svelte-i18n';

export const ssr = false;
export const prerender = true;

export const load = async () => {
	await waitLocale();
};
