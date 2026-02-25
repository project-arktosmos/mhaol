<script lang="ts">
	import QRCode from 'qrcode';
	import { fitsInQrCode } from '$utils/p2p/sdp-codec';

	export let data: string;
	export let size: number = 256;

	let dataUrl: string | null = null;
	let tooLarge = false;
	let error: string | null = null;

	$: tooLarge = !fitsInQrCode(data);

	$: if (data && !tooLarge) {
		generateQr(data);
	}

	async function generateQr(value: string) {
		try {
			dataUrl = await QRCode.toDataURL(value, {
				width: size,
				margin: 2,
				errorCorrectionLevel: 'L'
			});
			error = null;
		} catch {
			error = 'Failed to generate QR code';
			dataUrl = null;
		}
	}
</script>

<div class="flex flex-col items-center gap-2">
	{#if tooLarge}
		<div class="alert alert-warning text-sm">SDP too large for QR code. Use copy-paste instead.</div>
	{:else if error}
		<p class="text-sm text-error">{error}</p>
	{:else if dataUrl}
		<img src={dataUrl} alt="QR Code" class="rounded-lg" width={size} height={size} />
	{:else}
		<span class="loading loading-spinner loading-md"></span>
	{/if}
</div>
