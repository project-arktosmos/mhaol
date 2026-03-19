import '$services/i18n';
import { waitLocale } from 'svelte-i18n';

export const ssr = false;

export const load = async () => {
	await waitLocale();
};
