<script lang="ts">
	import classNames from 'classnames';
	import { onDestroy } from 'svelte';
	import { signalingChatService } from '$services/signaling-chat.service';
	import { signalingAdapter } from '$adapters/classes/signaling.adapter';
	import { contactHandshakeService } from 'webrtc/service';
	import { serverCatalogService } from '$services/server-catalog.service';
	import { playerService } from '$services/player.service';
	import PlayerVideo from '$components/player/PlayerVideo.svelte';
	import TmdbCatalogGrid from '$components/catalog/TmdbCatalogGrid.svelte';
	import type { PeerConnectionStatus, SignalingPeerInfo } from '$types/signaling.type';
	import type { ContactHandshakePhase } from 'webrtc/types';
	import type { DisplayTMDBMovie } from 'addons/tmdb/types';
	import type { CatalogMovie } from '$types/server-catalog.type';

	// Periodically request fresh catalog from all connected server peers
	const catalogRefreshInterval = setInterval(() => {
		const movies = serverCatalogService.state;
		let peerIds: string[] = [];
		movies.subscribe((s) => (peerIds = Object.keys(s.movies)))();
		for (const peerId of peerIds) {
			serverCatalogService.requestCatalog(peerId);
		}
	}, 10_000);
	onDestroy(() => clearInterval(catalogRefreshInterval));

	const chatStore = signalingChatService.state;
	const handshakeStore = contactHandshakeService.state;
	const catalogStore = serverCatalogService.state;
	const playerState = playerService.state;

	interface HandshakeStep {
		label: string;
		status: 'pending' | 'active' | 'waiting' | 'done' | 'error';
	}

	function getSteps(
		signalingPhase: string,
		activePeerId: string | null,
		peerConnectionStates: Record<string, PeerConnectionStatus>,
		peerPhases: Record<string, ContactHandshakePhase>,
		hasMovies: boolean
	): HandshakeStep[] {
		const steps: HandshakeStep[] = [];

		const signalingDone = signalingPhase === 'connected';
		const signalingError = signalingPhase === 'error';
		steps.push({
			label: 'Connect to signaling server',
			status: signalingError
				? 'error'
				: signalingDone
					? 'done'
					: signalingPhase === 'connecting'
						? 'active'
						: 'pending'
		});

		const hasPeers = activePeerId !== null;
		steps.push({
			label: 'Discover peers in room',
			status: !signalingDone ? 'pending' : hasPeers ? 'done' : 'waiting'
		});

		const peerStatus = activePeerId ? peerConnectionStates[activePeerId] : undefined;
		const peerConnected = peerStatus === 'connected';
		const peerFailed = peerStatus === 'failed';
		const peerConnecting = peerStatus === 'offering' || peerStatus === 'answering';
		steps.push({
			label: 'Establish WebRTC connection',
			status: !hasPeers
				? 'pending'
				: peerFailed
					? 'error'
					: peerConnected
						? 'done'
						: peerConnecting
							? 'active'
							: 'pending'
		});

		const phase = activePeerId ? peerPhases[activePeerId] : undefined;
		const handshakeDone = phase === 'accepted';
		const handshakeActive =
			phase === 'sending-request' ||
			phase === 'request-sent' ||
			phase === 'request-received' ||
			phase === 'sending-acceptance';
		steps.push({
			label: 'Exchange passports',
			status: !peerConnected
				? 'pending'
				: handshakeDone
					? 'done'
					: handshakeActive
						? 'active'
						: 'active'
		});

		steps.push({
			label: 'Receive server catalog',
			status: !handshakeDone ? 'pending' : hasMovies ? 'done' : 'active'
		});

		return steps;
	}

	function stableNumericId(str: string): number {
		let hash = 0;
		for (let i = 0; i < str.length; i++) {
			hash = (hash * 31 + str.charCodeAt(i)) | 0;
		}
		return Math.abs(hash);
	}

	function catalogToDisplayMovie(entry: CatalogMovie): DisplayTMDBMovie {
		const tmdb = entry.tmdb;
		const id = tmdb?.id ?? stableNumericId(entry.item.id);
		return {
			id,
			title: tmdb?.title ?? entry.item.name,
			originalTitle: tmdb?.originalTitle ?? entry.item.name,
			overview: tmdb?.overview ?? '',
			posterUrl: tmdb?.posterUrl ?? null,
			backdropUrl: tmdb?.backdropUrl ?? null,
			releaseYear: tmdb?.releaseYear ?? '',
			voteAverage: tmdb?.voteAverage ?? 0,
			voteCount: tmdb?.voteCount ?? 0,
			genres: tmdb?.genres ?? []
		};
	}

	let activePeerId = $derived($chatStore.activePeerId);

	// Flatten all catalog entries with their peerId
	let allCatalogEntries = $derived(
		Object.entries($catalogStore.movies).flatMap(([peerId, movies]) =>
			movies.map((entry) => ({ peerId, entry }))
		)
	);

	let displayMovies = $derived(
		[...allCatalogEntries]
			.sort((a, b) => (b.entry.streamable ? 1 : 0) - (a.entry.streamable ? 1 : 0))
			.map((e) => catalogToDisplayMovie(e.entry))
	);

	// IDs of items that are not streamable (shown at 50% opacity)
	let dimmedIds = $derived(
		new Set(
			allCatalogEntries
				.filter((e) => !e.entry.streamable)
				.map((e) => e.entry.tmdb?.id ?? stableNumericId(e.entry.item.id))
		)
	);

	// Derive aggregate phase and peerConnectionStates from all rooms
	let aggregatePhase = $derived.by(() => {
		const rooms = Object.values($chatStore.rooms);
		if (rooms.length === 0) return 'disconnected';
		if (rooms.some((r) => r.phase === 'connected')) return 'connected';
		if (rooms.some((r) => r.phase === 'connecting' || r.phase === 'authenticated'))
			return 'connecting';
		if (rooms.some((r) => r.phase === 'error')) return 'error';
		return 'disconnected';
	});

	let allPeerConnectionStates = $derived.by((): Record<string, PeerConnectionStatus> => {
		const result: Record<string, PeerConnectionStatus> = {};
		for (const room of Object.values($chatStore.rooms)) {
			Object.assign(result, room.peerConnectionStates);
		}
		return result;
	});

	let steps = $derived(
		getSteps(
			aggregatePhase,
			activePeerId,
			allPeerConnectionStates,
			$handshakeStore.peerPhases,
			displayMovies.length > 0
		)
	);

	let currentStepInfo = $derived.by(() => {
		const allDone = steps.every((s) => s.status === 'done');
		if (allDone) return { index: steps.length, label: 'Connected', status: 'done' as const };
		const errorIdx = steps.findIndex((s) => s.status === 'error');
		if (errorIdx !== -1)
			return { index: errorIdx + 1, label: steps[errorIdx].label, status: 'error' as const };
		const waitingIdx = steps.findIndex((s) => s.status === 'waiting');
		if (waitingIdx !== -1)
			return {
				index: waitingIdx + 1,
				label: steps[waitingIdx].label,
				status: 'waiting' as const
			};
		const activeIdx = steps.findIndex((s) => s.status === 'active');
		if (activeIdx !== -1)
			return { index: activeIdx + 1, label: steps[activeIdx].label, status: 'active' as const };
		return { index: 1, label: steps[0].label, status: 'pending' as const };
	});

	let isPlaying = $derived($playerState.currentFile !== null);

	function handleSelectMovie(movie: DisplayTMDBMovie) {
		if (dimmedIds.has(movie.id)) return;
		// Find the catalog entry matching this movie by TMDB ID or fallback hash
		const entry = allCatalogEntries.find((e) => {
			const tmdbLink = e.entry.item.links?.tmdb;
			if (tmdbLink && Number(tmdbLink.serviceId) === movie.id) return true;
			return stableNumericId(e.entry.item.id) === movie.id;
		});
		if (!entry) return;
		const tmdbLink = entry.entry.item.links?.tmdb;
		if (!tmdbLink) return;
		serverCatalogService.requestStream(entry.peerId, Number(tmdbLink.serviceId));
		window.scrollTo({ top: 0, behavior: 'smooth' });
	}

	let roomEntries = $derived(Object.entries($chatStore.rooms));

	let {
		onPeerClick,
		onPeerDisconnect
	}: {
		onPeerClick: (peerId: string) => void;
		onPeerDisconnect: (peerId: string) => void;
	} = $props();
</script>

<div class="flex h-full flex-col gap-4 overflow-y-auto">
	<!-- Player -->
	{#if isPlaying}
		<div class="card bg-base-200">
			<div class="card-body gap-2 p-4">
				<div class="flex items-center justify-between">
					<h2 class="card-title text-base">{$playerState.currentFile?.name ?? 'Player'}</h2>
					<button class="btn text-error btn-ghost btn-xs" onclick={() => playerService.stop()}>
						Close
					</button>
				</div>
				<PlayerVideo
					file={$playerState.currentFile}
					connectionState={$playerState.connectionState}
					positionSecs={$playerState.positionSecs}
					durationSecs={$playerState.durationSecs}
					buffering={$playerState.buffering}
				/>
			</div>
		</div>
	{/if}

	<!-- Connection Status Badge -->
	{#if currentStepInfo.status !== 'done'}
		<div
			class={classNames('badge gap-1.5 py-3', {
				'badge-primary': currentStepInfo.status === 'active',
				'badge-success': currentStepInfo.status === 'waiting',
				'badge-error': currentStepInfo.status === 'error',
				'badge-ghost': currentStepInfo.status === 'pending'
			})}
		>
			{#if currentStepInfo.status === 'active'}
				<span class="loading loading-xs loading-spinner"></span>
			{/if}
			<span>{currentStepInfo.index}/{steps.length}</span>
			<span>{currentStepInfo.label}</span>
		</div>
	{/if}

	<!-- Signaling Rooms (hidden once catalog is loaded) -->
	{#if displayMovies.length === 0}
		{#each roomEntries as [roomId, room] (roomId)}
			<div class="card bg-base-200">
				<div class="card-body gap-3 p-4">
					<div class="flex items-center justify-between">
						<h2 class="card-title text-base">
							{roomId.length > 12 ? signalingAdapter.shortAddress(roomId) : roomId}
						</h2>
						<span
							class={classNames('badge badge-sm', signalingAdapter.phaseBadgeClass(room.phase))}
						>
							{signalingAdapter.phaseLabel(room.phase)}
						</span>
					</div>
					{#if room.roomPeers.length > 0}
						<div class="flex flex-col gap-1">
							{#each room.roomPeers as peer (peer.peer_id)}
								{@const status = room.peerConnectionStates[peer.peer_id] ?? 'idle'}
								{@const handshakePhase = $handshakeStore.peerPhases[peer.peer_id]}
								{@const isActive = peer.peer_id === activePeerId}
								<div
									class={classNames(
										'flex cursor-pointer items-center gap-2 rounded-lg px-3 py-2 transition-colors',
										{
											'bg-primary/10': isActive,
											'hover:bg-base-300': !isActive
										}
									)}
									role="button"
									tabindex="0"
									onclick={() => onPeerClick(peer.peer_id)}
									onkeydown={(e: KeyboardEvent) => {
										if (e.key === 'Enter' || e.key === ' ') onPeerClick(peer.peer_id);
									}}
								>
									<span
										class={classNames('h-2.5 w-2.5 shrink-0 rounded-full', {
											'bg-base-content/30': status === 'idle',
											'animate-pulse bg-warning': status === 'offering' || status === 'answering',
											'bg-success': status === 'connected',
											'bg-error': status === 'failed'
										})}
									></span>
									<div class="min-w-0 flex-1">
										{#if peer.name}
											<div class="flex items-center gap-1.5">
												<span class="block truncate text-sm font-medium">{peer.name}</span>
												{#if peer.instance_type}
													<span class="badge badge-outline badge-xs">{peer.instance_type}</span>
												{/if}
											</div>
										{/if}
										<span class="block truncate font-mono text-xs">
											{signalingAdapter.shortAddress(peer.peer_id)}
										</span>
										<div class="flex items-center gap-1.5">
											<span
												class={classNames(
													'badge badge-xs',
													signalingAdapter.peerConnectionStatusBadgeClass(status)
												)}
											>
												{signalingAdapter.peerConnectionStatusLabel(status)}
											</span>
											{#if handshakePhase && handshakePhase !== 'idle'}
												<span class="text-[10px] text-base-content/50">
													{handshakePhase}
												</span>
											{/if}
										</div>
									</div>
									{#if status === 'connected'}
										<button
											class="btn text-error btn-ghost btn-xs"
											onclick={(e: MouseEvent) => {
												e.stopPropagation();
												onPeerDisconnect(peer.peer_id);
											}}
											title="Disconnect"
										>
											x
										</button>
									{/if}
								</div>
							{/each}
						</div>
					{:else}
						<span class="text-xs text-base-content/40">No peers in this room</span>
					{/if}
				</div>
			</div>
		{/each}
	{/if}

	<!-- Movie Catalog Grid -->
	{#if displayMovies.length > 0}
		<div>
			<h2 class="mb-3 text-lg font-semibold">Server Movies</h2>
			<TmdbCatalogGrid movies={displayMovies} {dimmedIds} onselectMovie={handleSelectMovie} />
		</div>
	{/if}

	<!-- Local Identity -->
	{#if $chatStore.localPeerId}
		<div class="text-center text-[10px] text-base-content/30">
			You: <span class="font-mono">{signalingAdapter.shortAddress($chatStore.localPeerId)}</span>
		</div>
	{/if}
</div>
