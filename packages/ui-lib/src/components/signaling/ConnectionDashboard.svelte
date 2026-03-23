<script lang="ts">
	import classNames from 'classnames';
	import { signalingChatService } from 'ui-lib/services/signaling-chat.service';
	import { signalingAdapter } from 'ui-lib/adapters/classes/signaling.adapter';
	import { contactHandshakeService } from 'webrtc/service';
	import { serverCatalogService } from 'ui-lib/services/server-catalog.service';
	import { playerService } from 'ui-lib/services/player.service';
	import PlayerVideo from 'ui-lib/components/player/PlayerVideo.svelte';
	import TmdbBrowseGrid from 'ui-lib/components/tmdb-browse/TmdbBrowseGrid.svelte';
	import type { PeerConnectionStatus, SignalingPeerInfo } from 'ui-lib/types/signaling.type';
	import type { ContactHandshakePhase } from 'webrtc/types';
	import type { DisplayTMDBMovie } from 'addons/tmdb/types';
	import type { CatalogMovie } from 'ui-lib/types/server-catalog.type';

	const chatStore = signalingChatService.state;
	const handshakeStore = contactHandshakeService.state;
	const catalogStore = serverCatalogService.state;
	const playerState = playerService.state;

	interface HandshakeStep {
		label: string;
		status: 'pending' | 'active' | 'done' | 'error';
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
			status: !signalingDone ? 'pending' : hasPeers ? 'done' : 'active'
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

	function catalogToDisplayMovie(entry: CatalogMovie): DisplayTMDBMovie {
		const tmdb = entry.tmdb;
		return {
			id: 0, // overridden by displayMovies with array index
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

	// Flatten all catalog entries with their peerId, using array index as stable ID
	let allCatalogEntries = $derived(
		Object.entries($catalogStore.movies).flatMap(([peerId, movies]) =>
			movies.map((entry) => ({ peerId, entry }))
		)
	);

	let displayMovies = $derived(
		allCatalogEntries.map((e, i) => {
			const movie = catalogToDisplayMovie(e.entry);
			return { ...movie, id: i };
		})
	);

	let steps = $derived(
		getSteps(
			$chatStore.phase,
			activePeerId,
			$chatStore.peerConnectionStates,
			$handshakeStore.peerPhases,
			displayMovies.length > 0
		)
	);

	let isPlaying = $derived($playerState.currentFile !== null);

	function handleSelectMovie(movie: DisplayTMDBMovie) {
		const entry = allCatalogEntries[movie.id];
		if (!entry) {
			console.error('[ConnectionDashboard] No catalog entry for movie.id:', movie.id);
			return;
		}
		console.log('[ConnectionDashboard] Stream request:', {
			movieId: movie.id,
			title: movie.title,
			itemId: entry.entry.item.id,
			itemPath: entry.entry.item.path,
			itemName: entry.entry.item.name
		});
		serverCatalogService.requestStream(entry.peerId, entry.entry.item.path);
	}

	let {
		roomPeers = [],
		peerConnectionStates = {},
		onPeerClick,
		onPeerDisconnect
	}: {
		roomPeers: SignalingPeerInfo[];
		peerConnectionStates: Record<string, PeerConnectionStatus>;
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

	<!-- Handshake Steps -->
	<div class="card bg-base-200">
		<div class="card-body gap-3 p-4">
			<h2 class="card-title text-base">Connection Steps</h2>
			<ul class="steps steps-vertical">
				{#each steps as step, i (i)}
					<li
						class={classNames('step', {
							'step-success': step.status === 'done',
							'step-primary': step.status === 'active',
							'step-error': step.status === 'error'
						})}
					>
						<div class="flex items-center gap-2 text-left text-sm">
							{#if step.status === 'active'}
								<span class="loading loading-xs loading-spinner"></span>
							{/if}
							<span
								class={classNames({
									'text-base-content/40': step.status === 'pending',
									'text-error': step.status === 'error'
								})}
							>
								{step.label}
							</span>
						</div>
					</li>
				{/each}
			</ul>
		</div>
	</div>

	<!-- Peers in Room -->
	{#if roomPeers.length > 0}
		<div class="card bg-base-200">
			<div class="card-body gap-3 p-4">
				<div class="flex items-center justify-between">
					<h2 class="card-title text-base">Peers</h2>
					<span class="badge badge-ghost badge-sm">{roomPeers.length}</span>
				</div>
				<div class="flex flex-col gap-1">
					{#each roomPeers as peer (peer.peer_id)}
						{@const status = peerConnectionStates[peer.peer_id] ?? 'idle'}
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
								<span class="text-[10px] text-base-content/40">
									{signalingAdapter.peerConnectionStatusLabel(status)}
								</span>
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
			</div>
		</div>
	{/if}

	<!-- Movie Catalog Grid -->
	{#if displayMovies.length > 0}
		<div>
			<h2 class="mb-3 text-lg font-semibold">Server Movies</h2>
			<TmdbBrowseGrid movies={displayMovies} onselectMovie={handleSelectMovie} />
		</div>
	{/if}

	<!-- Local Identity -->
	{#if $chatStore.localPeerId}
		<div class="text-center text-[10px] text-base-content/30">
			You: <span class="font-mono">{signalingAdapter.shortAddress($chatStore.localPeerId)}</span>
		</div>
	{/if}
</div>
