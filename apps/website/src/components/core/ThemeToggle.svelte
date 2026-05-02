<script lang="ts">
	import { onMount } from 'svelte';
	import { Icon } from 'cloud-ui';

	const STORAGE_KEY = 'mhaol-website:theme';

	let theme = $state<'light' | 'dark'>('light');

	function apply(next: 'light' | 'dark') {
		theme = next;
		document.documentElement.setAttribute('data-theme', next);
		try {
			localStorage.setItem(STORAGE_KEY, next);
		} catch {
			/* ignore */
		}
	}

	onMount(() => {
		let initial: 'light' | 'dark' = 'light';
		try {
			const stored = localStorage.getItem(STORAGE_KEY);
			if (stored === 'light' || stored === 'dark') {
				initial = stored;
			} else if (window.matchMedia?.('(prefers-color-scheme: dark)').matches) {
				initial = 'dark';
			}
		} catch {
			/* ignore */
		}
		apply(initial);
	});

	function toggle() {
		apply(theme === 'dark' ? 'light' : 'dark');
	}
</script>

<button
	type="button"
	class="btn btn-circle btn-ghost btn-sm"
	onclick={toggle}
	title={theme === 'dark' ? 'Switch to light theme' : 'Switch to dark theme'}
	aria-label="Toggle theme"
>
	{#if theme === 'dark'}
		<Icon name="lorc/sun" size={18} />
	{:else}
		<Icon name="lorc/moon" size={18} />
	{/if}
</button>
