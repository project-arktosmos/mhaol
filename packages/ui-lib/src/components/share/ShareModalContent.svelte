<script lang="ts">
	import { onMount } from 'svelte';
	import { apiUrl } from 'frontend/lib/api-base';
	import QRCode from 'qrcode';

	let shareUrl = $state('');
	let qrDataUrl = $state('');
	let copied = $state(false);
	let loading = $state(true);

	onMount(async () => {
		try {
			const res = await fetch(apiUrl('/api/network/info'));
			const data: { local_ip: string | null } = await res.json();

			if (data.local_ip) {
				const loc = window.location;
				shareUrl = `${loc.protocol}//${data.local_ip}:${loc.port}${loc.pathname}${loc.search}${loc.hash}`;
			} else {
				shareUrl = window.location.href;
			}

			qrDataUrl = await QRCode.toDataURL(shareUrl, {
				width: 256,
				margin: 2,
				color: { dark: '#000000', light: '#ffffff' }
			});
		} catch {
			shareUrl = window.location.href;
			qrDataUrl = await QRCode.toDataURL(shareUrl, {
				width: 256,
				margin: 2,
				color: { dark: '#000000', light: '#ffffff' }
			});
		} finally {
			loading = false;
		}
	});

	async function copyUrl() {
		await navigator.clipboard.writeText(shareUrl);
		copied = true;
		setTimeout(() => (copied = false), 2000);
	}
</script>

<div class="pr-8">
	<h3 class="text-lg font-bold">Share</h3>
	<p class="text-sm text-base-content/60">Share this page with devices on your local network</p>
</div>

{#if loading}
	<div class="mt-6 flex justify-center">
		<span class="loading loading-spinner loading-lg"></span>
	</div>
{:else}
	<div class="mt-4 flex flex-col items-center gap-4">
		<div class="rounded-lg bg-white p-2">
			<img src={qrDataUrl} alt="QR code for share URL" class="h-64 w-64" />
		</div>

		<div class="flex w-full items-center gap-2">
			<input
				type="text"
				readonly
				value={shareUrl}
				class="input input-bordered w-full font-mono text-sm"
			/>
			<button class="btn btn-primary btn-sm whitespace-nowrap" onclick={copyUrl}>
				{copied ? 'Copied!' : 'Copy'}
			</button>
		</div>

		<p class="text-xs text-base-content/50">
			Scan the QR code or open the URL on another device connected to the same network.
		</p>
	</div>
{/if}
