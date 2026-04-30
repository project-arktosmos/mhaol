<script lang="ts">
	import { onDestroy } from 'svelte';
	import Modal from 'ui-lib/components/core/Modal.svelte';
	import type { EmulatorCore } from 'ui-lib/components/videogames/emulator-cores';

	// EmulatorJS bootstraps via a small loader script that reads a handful of
	// `EJS_*` globals off `window`, then injects the emulator into the element
	// pointed to by `EJS_player`. https://emulatorjs.org/docs4devs/options
	//
	// The standard CDN is stable, MIT-licensed and supports a long list of
	// retro cores (Gambatte, mGBA, FCEUmm, snes9x, mupen64plus_next, ...).
	// The full data + per-core WASM blobs are fetched from the same CDN on
	// demand. We don't bundle anything ourselves.
	const EJS_CDN_BASE = 'https://cdn.emulatorjs.org/stable/data/';

	interface Props {
		open: boolean;
		core: EmulatorCore | null;
		gameUrl: string | null;
		gameName?: string;
		onclose: () => void;
	}

	let { open, core, gameUrl, gameName = 'Game', onclose }: Props = $props();

	let containerEl = $state<HTMLDivElement | null>(null);
	const containerId = `emulator-${Math.random().toString(36).slice(2, 10)}`;

	let scriptEl: HTMLScriptElement | null = null;
	let booted = false;

	const EJS_GLOBAL_KEYS = [
		'EJS_player',
		'EJS_core',
		'EJS_gameUrl',
		'EJS_gameName',
		'EJS_pathtodata',
		'EJS_startOnLoaded',
		'EJS_emulator',
		'EJS_Buttons',
		'EJS_volume'
	] as const;

	function clearGlobals() {
		const w = window as unknown as Record<string, unknown>;
		for (const key of EJS_GLOBAL_KEYS) {
			try {
				delete w[key];
			} catch {
				w[key] = undefined;
			}
		}
	}

	function teardown() {
		const w = window as unknown as { EJS_emulator?: { exit?: () => void } };
		try {
			w.EJS_emulator?.exit?.();
		} catch {
			// ignore — exit() can throw if the emulator hasn't fully initialised
		}
		if (scriptEl && scriptEl.parentNode) {
			scriptEl.parentNode.removeChild(scriptEl);
		}
		scriptEl = null;
		clearGlobals();
		if (containerEl) containerEl.innerHTML = '';
		booted = false;
	}

	function boot() {
		if (booted) return;
		if (!containerEl || !core || !gameUrl) return;
		booted = true;
		const w = window as unknown as Record<string, unknown>;
		w['EJS_player'] = `#${containerId}`;
		w['EJS_core'] = core;
		w['EJS_gameUrl'] = gameUrl;
		w['EJS_gameName'] = gameName;
		w['EJS_pathtodata'] = EJS_CDN_BASE;
		w['EJS_startOnLoaded'] = true;
		const script = document.createElement('script');
		script.src = `${EJS_CDN_BASE}loader.js`;
		script.async = true;
		document.body.appendChild(script);
		scriptEl = script;
	}

	$effect(() => {
		if (open && containerEl && core && gameUrl) {
			boot();
		} else if (!open && booted) {
			teardown();
		}
	});

	onDestroy(() => {
		teardown();
	});

	function handleClose() {
		teardown();
		onclose();
	}
</script>

<Modal {open} maxWidth="max-w-5xl" onclose={handleClose}>
	<div class="flex flex-col gap-3">
		<header class="flex items-center justify-between gap-2 pr-8">
			<h2 class="text-lg font-semibold [overflow-wrap:anywhere]">{gameName}</h2>
			{#if core}
				<span class="badge badge-outline badge-sm">{core}</span>
			{/if}
		</header>
		{#if !core}
			<p class="text-sm text-error">No emulator core is wired up for this game yet.</p>
		{:else if !gameUrl}
			<p class="text-sm text-base-content/60">Loading ROM…</p>
		{:else}
			<div
				id={containerId}
				bind:this={containerEl}
				class="aspect-video w-full overflow-hidden rounded-box bg-black"
			></div>
			<p class="text-xs text-base-content/60">
				Powered by EmulatorJS (cdn.emulatorjs.org). Click the canvas, then press a key to start.
			</p>
		{/if}
	</div>
</Modal>
