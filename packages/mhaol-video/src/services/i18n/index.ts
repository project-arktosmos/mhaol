import { register, init, getLocaleFromNavigator } from 'svelte-i18n';

// Register locales
register('en', () => import('./locales/en.json'));
register('qq', () => import('./locales/qq.json'));

// Initialize i18n
init({
	fallbackLocale: 'en',
	initialLocale: getLocaleFromNavigator()
});
