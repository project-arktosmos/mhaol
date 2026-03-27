<script lang="ts">
	import { onMount } from 'svelte';
	import { fetchRaw } from 'ui-lib/transport/fetch-helpers';
	import { p2pStreamService } from 'ui-lib/services/p2p-stream.service';
	import { P2P_VIDEO_CODEC_OPTIONS, P2P_VIDEO_QUALITY_OPTIONS } from 'ui-lib/types/p2p-stream.type';
	import type { P2pVideoCodec, P2pVideoQuality, P2pStreamMode } from 'ui-lib/types/p2p-stream.type';
	import classNames from 'classnames';

	const settings = p2pStreamService.store;

	let resetting = $state(false);
	let error = $state<string | null>(null);

	onMount(() => {
		p2pStreamService.initialize();
	});

	function handleModeChange(mode: P2pStreamMode) {
		p2pStreamService.setDefaultStreamMode(mode);
	}

	function handleVideoCodecChange(event: Event) {
		const target = event.target as HTMLSelectElement;
		p2pStreamService.setVideoCodec(target.value as P2pVideoCodec);
	}

	function handleVideoQualityChange(event: Event) {
		const target = event.target as HTMLSelectElement;
		p2pStreamService.setVideoQuality(target.value as P2pVideoQuality);
	}

	async function handleReset() {
		resetting = true;
		error = null;

		try {
			const res = await fetchRaw('/api/database/reset', { method: 'POST' });
			if (!res.ok) {
				const body = await res.json().catch(() => ({}));
				throw new Error((body as { error?: string }).error ?? `HTTP ${res.status}`);
			}
			window.location.reload();
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
			resetting = false;
		}
	}
</script>

<div class="pr-8">
	<h3 class="text-lg font-bold">Settings</h3>
	<p class="text-sm text-base-content/60">Application configuration and maintenance</p>
</div>

{#if error}
	<div class="mt-4 alert alert-error">
		<span>{error}</span>
		<button class="btn btn-ghost btn-sm" onclick={() => (error = null)}>x</button>
	</div>
{/if}

<div class="card mt-4 bg-base-200">
	<div class="card-body gap-4">
		<h2 class="card-title text-lg">Video</h2>

		<!-- Default Stream Mode Toggle -->
		<div class="form-control">
			<span class="label">
				<span class="label-text">Default Stream Mode</span>
			</span>
			<div class="join w-full">
				<button
					class={classNames('btn join-item flex-1', {
						'btn-primary': $settings.defaultStreamMode === 'video',
						'btn-ghost': $settings.defaultStreamMode !== 'video'
					})}
					onclick={() => handleModeChange('video')}
				>
					Video + Audio
				</button>
				<button
					class={classNames('btn join-item flex-1', {
						'btn-primary': $settings.defaultStreamMode === 'audio',
						'btn-ghost': $settings.defaultStreamMode !== 'audio'
					})}
					onclick={() => handleModeChange('audio')}
				>
					Audio Only
				</button>
			</div>
		</div>

		<!-- Stream Quality -->
		<div class="form-control">
			<label class="label" for="video-quality-select">
				<span class="label-text">Stream Quality</span>
			</label>
			<select
				id="video-quality-select"
				class="select-bordered select w-full"
				value={$settings.videoQuality}
				onchange={handleVideoQualityChange}
			>
				{#each P2P_VIDEO_QUALITY_OPTIONS as option}
					<option value={option.value}>
						{option.label} - {option.description}
					</option>
				{/each}
			</select>
			<label class="label" for="video-quality-select">
				<span class="label-text-alt text-base-content/50">
					Controls resolution and bitrate. Lower quality uses less bandwidth.
				</span>
			</label>
		</div>

		<!-- Video Codec -->
		<div class="form-control">
			<label class="label" for="video-codec-select">
				<span class="label-text">Video Codec</span>
			</label>
			<select
				id="video-codec-select"
				class="select-bordered select w-full"
				value={$settings.videoCodec}
				onchange={handleVideoCodecChange}
			>
				{#each P2P_VIDEO_CODEC_OPTIONS as option}
					<option value={option.value}>
						{option.label} - {option.description}
					</option>
				{/each}
			</select>
		</div>

		<!-- Audio Codec -->
		<div class="form-control">
			<label class="label" for="audio-codec-select">
				<span class="label-text">Audio Codec</span>
			</label>
			<select
				id="audio-codec-select"
				class="select-bordered select w-full"
				disabled
				value={$settings.audioCodec}
			>
				<option value="opus">Opus - Only supported codec</option>
			</select>
		</div>
	</div>
</div>

<div class="card mt-4 bg-base-200">
	<div class="card-body">
		<h2 class="card-title text-lg text-error">Danger Zone</h2>

		<div class="mt-2 flex items-center justify-between rounded-lg border border-error/30 p-4">
			<div>
				<h3 class="font-semibold">Reset Database</h3>
				<p class="text-sm opacity-70">
					Drop all tables, recreate from schema, and reseed defaults.
				</p>
			</div>
			<button class="btn btn-sm btn-error" disabled={resetting} onclick={handleReset}>
				{#if resetting}
					<span class="loading loading-sm loading-spinner"></span>
				{:else}
					Reset Database
				{/if}
			</button>
		</div>
	</div>
</div>
