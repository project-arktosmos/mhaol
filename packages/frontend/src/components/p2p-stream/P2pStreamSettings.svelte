<script lang="ts">
	import classNames from 'classnames';
	import { p2pStreamService } from '$services/p2p-stream.service';
	import { P2P_VIDEO_CODEC_OPTIONS, P2P_VIDEO_QUALITY_OPTIONS } from '$types/p2p-stream.type';
	import type { P2pVideoCodec, P2pVideoQuality, P2pStreamMode } from '$types/p2p-stream.type';

	const settings = p2pStreamService.store;
	const state = p2pStreamService.state;

	// TURN server input
	let newTurnServer = '';
	let showAdvanced = false;

	// Debounced STUN server save
	let stunTimeout: ReturnType<typeof setTimeout> | null = null;

	function handleStunChange(event: Event) {
		const value = (event.target as HTMLInputElement).value;
		if (stunTimeout) clearTimeout(stunTimeout);
		stunTimeout = setTimeout(() => {
			p2pStreamService.setStunServer(value);
		}, 500);
	}

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

	function handleAddTurnServer() {
		const url = newTurnServer.trim();
		if (!url) return;
		p2pStreamService.addTurnServer(url);
		newTurnServer = '';
	}

	function handleRemoveTurnServer(url: string) {
		p2pStreamService.removeTurnServer(url);
	}

	function handleRefreshHealth() {
		p2pStreamService.checkHealth();
	}

	function handleTurnKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter') {
			event.preventDefault();
			handleAddTurnServer();
		}
	}
</script>

<div class="card bg-base-200">
	<div class="card-body gap-4">
		<h2 class="card-title text-lg">P2P Streaming</h2>

		<!-- Server Status -->
		<div
			class={classNames('rounded-lg p-3', {
				'bg-success/10': $state.serverAvailable,
				'bg-warning/10': !$state.serverAvailable
			})}
		>
			<div class="flex items-center justify-between">
				<div class="flex items-center gap-2">
					<div
						class={classNames('h-2 w-2 rounded-full', {
							'bg-success': $state.serverAvailable,
							'bg-warning': !$state.serverAvailable
						})}
					></div>
					<span class="text-sm font-medium">
						{#if $state.serverAvailable}
							Stream Server Running
						{:else}
							Stream Server Not Available
						{/if}
					</span>
				</div>
				<button class="btn btn-ghost btn-xs" on:click={handleRefreshHealth} title="Refresh status">
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="h-4 w-4"
						fill="none"
						viewBox="0 0 24 24"
						stroke="currentColor"
						stroke-width="2"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
						/>
					</svg>
				</button>
			</div>
		</div>

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
					on:click={() => handleModeChange('video')}
				>
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="h-4 w-4"
						fill="none"
						viewBox="0 0 24 24"
						stroke="currentColor"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M15 10l4.553-2.276A1 1 0 0121 8.618v6.764a1 1 0 01-1.447.894L15 14M5 18h8a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z"
						/>
					</svg>
					Video + Audio
				</button>
				<button
					class={classNames('btn join-item flex-1', {
						'btn-primary': $settings.defaultStreamMode === 'audio',
						'btn-ghost': $settings.defaultStreamMode !== 'audio'
					})}
					on:click={() => handleModeChange('audio')}
				>
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="h-4 w-4"
						fill="none"
						viewBox="0 0 24 24"
						stroke="currentColor"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3"
						/>
					</svg>
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
				on:change={handleVideoQualityChange}
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
				on:change={handleVideoCodecChange}
			>
				{#each P2P_VIDEO_CODEC_OPTIONS as option}
					<option value={option.value}>
						{option.label} - {option.description}
					</option>
				{/each}
			</select>
			<label class="label" for="video-codec-select">
				<span class="label-text-alt text-base-content/50">
					Applied server-side when streaming sessions are created
				</span>
			</label>
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

		<!-- STUN Server -->
		<div class="form-control">
			<label class="label" for="stun-server-input">
				<span class="label-text">STUN Server</span>
			</label>
			<input
				id="stun-server-input"
				type="text"
				class="input-bordered input w-full font-mono text-sm"
				placeholder="stun:stun.l.google.com:19302"
				value={$settings.stunServer}
				on:input={handleStunChange}
			/>
			<label class="label" for="stun-server-input">
				<span class="label-text-alt text-base-content/50">
					Used for ICE connectivity (NAT traversal)
				</span>
			</label>
		</div>

		<!-- Advanced: TURN Servers (Collapsible) -->
		<div class="divider my-1"></div>
		<button
			class="flex w-full items-center justify-between text-sm text-base-content/70 hover:text-base-content"
			on:click={() => (showAdvanced = !showAdvanced)}
		>
			<span>TURN Servers</span>
			<svg
				xmlns="http://www.w3.org/2000/svg"
				class="h-4 w-4 transition-transform"
				class:rotate-180={showAdvanced}
				fill="none"
				viewBox="0 0 24 24"
				stroke="currentColor"
			>
				<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
			</svg>
		</button>

		{#if showAdvanced}
			<div class="mt-2 flex flex-col gap-3 rounded-lg bg-base-300 p-3">
				<p class="text-xs text-base-content/60">
					TURN servers relay traffic when direct peer-to-peer connections fail. Add server URLs in
					the format <code class="text-xs">turn:host:port</code> or
					<code class="text-xs">turns:host:port</code>.
				</p>

				<!-- Existing TURN servers -->
				{#if $settings.turnServers.length > 0}
					<div class="flex flex-col gap-2">
						{#each $settings.turnServers as server}
							<div class="flex items-center justify-between rounded-lg bg-base-200 px-3 py-2">
								<span class="font-mono text-sm">{server}</span>
								<button
									class="btn text-error btn-ghost btn-xs"
									on:click={() => handleRemoveTurnServer(server)}
									title="Remove server"
								>
									<svg
										xmlns="http://www.w3.org/2000/svg"
										class="h-4 w-4"
										fill="none"
										viewBox="0 0 24 24"
										stroke="currentColor"
										stroke-width="2"
									>
										<path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
									</svg>
								</button>
							</div>
						{/each}
					</div>
				{:else}
					<p class="text-center text-xs text-base-content/40">No TURN servers configured</p>
				{/if}

				<!-- Add TURN server -->
				<div class="flex items-center gap-2">
					<input
						type="text"
						class="input-bordered input input-sm flex-1 font-mono text-xs"
						placeholder="turn:example.com:3478"
						bind:value={newTurnServer}
						on:keydown={handleTurnKeydown}
					/>
					<button
						class="btn btn-sm btn-primary"
						on:click={handleAddTurnServer}
						disabled={!newTurnServer.trim()}
					>
						Add
					</button>
				</div>
			</div>
		{/if}

		<!-- Error display -->
		{#if $state.error}
			<div class="alert text-sm alert-error">
				<span>{$state.error}</span>
			</div>
		{/if}
	</div>
</div>
