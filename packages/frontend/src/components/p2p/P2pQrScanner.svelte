<script lang="ts">
	import { onMount, onDestroy, createEventDispatcher } from 'svelte';
	import { Html5Qrcode } from 'html5-qrcode';

	const dispatch = createEventDispatcher<{
		scan: { data: string };
		cancel: void;
	}>();

	const scannerId =
		'p2p-qr-scanner-' +
		(typeof crypto.randomUUID === 'function'
			? crypto.randomUUID().slice(0, 8)
			: Math.random().toString(36).slice(2, 10));
	let scanner: Html5Qrcode | null = null;
	let error: string | null = null;
	let scanning = false;

	onMount(async () => {
		try {
			scanner = new Html5Qrcode(scannerId);
			await scanner.start(
				{ facingMode: 'environment' },
				{ fps: 10, qrbox: { width: 250, height: 250 } },
				(decodedText) => {
					dispatch('scan', { data: decodedText });
					stopScanning();
				},
				() => {
					// No QR found in frame
				}
			);
			scanning = true;
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to start camera';
		}
	});

	onDestroy(() => {
		stopScanning();
	});

	async function stopScanning() {
		if (scanner && scanning) {
			try {
				await scanner.stop();
			} catch {
				// Ignore stop errors
			}
			scanning = false;
		}
	}

	function handleCancel() {
		stopScanning();
		dispatch('cancel');
	}
</script>

<div class="flex flex-col items-center gap-3">
	<div id={scannerId} class="w-full max-w-xs overflow-hidden rounded-lg"></div>
	{#if error}
		<p class="text-sm text-error">{error}</p>
	{/if}
	<button class="btn btn-ghost btn-sm" on:click={handleCancel}>Cancel Scan</button>
</div>
