<script lang="ts">
	import { onMount, onDestroy } from 'svelte';

	interface Props {
		romUrl: string;
		core: string;
		gameName: string;
		onclose: () => void;
	}

	let { romUrl, core, gameName, onclose }: Props = $props();

	let containerEl = $state<HTMLDivElement | null>(null);
	let scriptEl: HTMLScriptElement | null = null;

	onMount(() => {
		(window as any).EJS_player = '#emulator-game';
		(window as any).EJS_gameUrl = romUrl;
		(window as any).EJS_core = core;
		(window as any).EJS_gameName = gameName;
		(window as any).EJS_pathtodata = 'https://cdn.emulatorjs.org/stable/data/';

		scriptEl = document.createElement('script');
		scriptEl.src = 'https://cdn.emulatorjs.org/stable/data/loader.js';
		document.body.appendChild(scriptEl);
	});

	onDestroy(() => {
		if (scriptEl) {
			scriptEl.remove();
		}
		delete (window as any).EJS_player;
		delete (window as any).EJS_gameUrl;
		delete (window as any).EJS_core;
		delete (window as any).EJS_gameName;
		delete (window as any).EJS_pathtodata;
		delete (window as any).EJS_emulator;

		const ejsElements = document.querySelectorAll('[id^="ejs_"]');
		ejsElements.forEach((el) => el.remove());
	});
</script>

<div class="fixed inset-0 z-50 flex flex-col bg-black">
	<div class="flex items-center justify-between px-4 py-2">
		<span class="text-sm font-medium text-white">{gameName}</span>
		<button
			class="btn btn-circle text-white btn-ghost btn-sm"
			onclick={onclose}
			aria-label="Close emulator"
		>
			<svg
				xmlns="http://www.w3.org/2000/svg"
				class="h-5 w-5"
				fill="none"
				viewBox="0 0 24 24"
				stroke="currentColor"
				stroke-width="2"
			>
				<path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
			</svg>
		</button>
	</div>
	<div id="emulator-game" class="flex-1" bind:this={containerEl}></div>
</div>
