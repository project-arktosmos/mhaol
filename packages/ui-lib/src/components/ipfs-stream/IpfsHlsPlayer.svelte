<script lang="ts">
	import { onDestroy } from 'svelte';
	import HlsPlayer from 'ui-lib/components/player/HlsPlayer.svelte';

	interface Props {
		cid: string;
		onClose?: () => void;
	}

	let { cid, onClose }: Props = $props();

	let sessionId = $state<string | null>(null);
	let playlistUrl = $state<string | null>(null);
	let error = $state<string | null>(null);
	let starting = $state(false);

	async function start(targetCid: string): Promise<void> {
		starting = true;
		error = null;
		try {
			const res = await fetch('/api/ipfs-stream/sessions', {
				method: 'POST',
				headers: { 'content-type': 'application/json' },
				body: JSON.stringify({ cid: targetCid })
			});
			if (!res.ok) {
				let message = `HTTP ${res.status}`;
				try {
					const body = await res.json();
					if (body && typeof body.error === 'string') message = body.error;
				} catch {
					// ignore
				}
				throw new Error(message);
			}
			const body = (await res.json()) as {
				sessionId: string;
				playlistUrl: string;
				playlistReady: boolean;
			};
			sessionId = body.sessionId;
			playlistUrl = body.playlistUrl;
		} catch (err) {
			error = err instanceof Error ? err.message : 'Unknown error';
		} finally {
			starting = false;
		}
	}

	async function stop(targetSessionId: string): Promise<void> {
		try {
			await fetch(`/api/ipfs-stream/sessions/${encodeURIComponent(targetSessionId)}`, {
				method: 'DELETE'
			});
		} catch {
			// best-effort: the cloud will gc on shutdown
		}
	}

	$effect(() => {
		const targetCid = cid;
		if (!targetCid) return;
		void start(targetCid);
	});

	onDestroy(() => {
		const id = sessionId;
		if (id) void stop(id);
	});
</script>

<div class="flex flex-col gap-3">
	{#if starting && !playlistUrl}
		<div class="flex items-center justify-center gap-3 rounded-lg bg-base-200 p-6">
			<span class="loading loading-spinner text-primary"></span>
			<span class="text-sm">Starting HLS stream over IPFS…</span>
		</div>
	{/if}

	{#if error}
		<div class="alert alert-error">
			<span>{error}</span>
		</div>
	{/if}

	{#if playlistUrl}
		<HlsPlayer src={playlistUrl} />
	{/if}

	{#if onClose}
		<div class="flex justify-end">
			<button type="button" class="btn btn-outline btn-sm" onclick={onClose}>Close</button>
		</div>
	{/if}
</div>
