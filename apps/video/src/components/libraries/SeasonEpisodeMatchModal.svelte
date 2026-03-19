<script lang="ts">
	import type { LibraryFile } from '$types/library.type';
	import type { TMDBEpisode } from 'addons/tmdb/types';

	interface EpisodeMatch {
		file: LibraryFile;
		seasonNumber: number;
		episodeNumber: number;
		episodeName: string;
		matched: boolean;
	}

	interface Props {
		tmdbId: number;
		seasonNumber: number;
		showName: string;
		seasonName: string;
		files: LibraryFile[];
		onlinkall: (
			matches: Array<{ file: LibraryFile; seasonNumber: number; episodeNumber: number }>
		) => void;
		onclose: () => void;
	}

	let { tmdbId, seasonNumber, showName, seasonName, files, onlinkall, onclose }: Props = $props();

	let loading = $state(true);
	let error: string | null = $state(null);
	let episodeMatches: EpisodeMatch[] = $state([]);

	function parseEpisodeFromFilename(name: string): { season: number; episode: number } | null {
		const match = name.match(/[Ss](\d{1,2})[Ee](\d{1,2})/);
		if (match) {
			return { season: parseInt(match[1], 10), episode: parseInt(match[2], 10) };
		}
		return null;
	}

	async function loadMatches() {
		loading = true;
		error = null;
		try {
			const res = await fetch(`/api/tmdb/tv/${tmdbId}/season/${seasonNumber}`);
			if (!res.ok) {
				error = 'Failed to fetch season episodes';
				return;
			}
			const data = await res.json();
			const episodes: TMDBEpisode[] = data.episodes ?? [];
			const episodeMap = new Map<number, TMDBEpisode>();
			for (const ep of episodes) {
				episodeMap.set(ep.episode_number, ep);
			}

			episodeMatches = files.map((f) => {
				const parsed = parseEpisodeFromFilename(f.name);
				if (!parsed || parsed.season !== seasonNumber) {
					return { file: f, seasonNumber, episodeNumber: 0, episodeName: '', matched: false };
				}
				const ep = episodeMap.get(parsed.episode);
				return {
					file: f,
					seasonNumber,
					episodeNumber: parsed.episode,
					episodeName: ep?.name ?? '',
					matched: ep !== undefined
				};
			});
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			loading = false;
		}
	}

	$effect(() => {
		loadMatches();
	});

	let matched = $derived(episodeMatches.filter((m) => m.matched));
	let unmatched = $derived(episodeMatches.filter((m) => !m.matched));

	function confirm() {
		onlinkall(
			matched.map((m) => ({
				file: m.file,
				seasonNumber: m.seasonNumber,
				episodeNumber: m.episodeNumber
			}))
		);
	}
</script>

<div class="modal-open modal">
	<div class="modal-box max-w-2xl">
		<button class="btn absolute top-2 right-2 btn-circle btn-ghost btn-sm" onclick={onclose}>
			&times;
		</button>

		<h3 class="text-lg font-bold">Match Episodes</h3>
		<p class="mt-1 text-sm opacity-60">
			{showName} · {seasonName}
		</p>

		{#if loading}
			<div class="flex justify-center py-10">
				<span class="loading loading-md loading-spinner"></span>
			</div>
		{:else if error}
			<div class="mt-4 rounded-lg bg-error/10 px-3 py-2 text-sm text-error">{error}</div>
			<div class="mt-3 flex justify-end">
				<button class="btn btn-ghost btn-sm" onclick={onclose}>Close</button>
			</div>
		{:else}
			<div class="mt-4 max-h-80 overflow-y-auto rounded-lg bg-base-100">
				<table class="table w-full table-xs">
					<thead class="sticky top-0 bg-base-100">
						<tr>
							<th>File</th>
							<th class="w-20">Episode</th>
							<th class="w-32">Title</th>
							<th class="w-16">Status</th>
						</tr>
					</thead>
					<tbody>
						{#each episodeMatches as m (m.file.path)}
							<tr>
								<td class="max-w-xs truncate font-mono text-xs opacity-70" title={m.file.name}>
									{m.file.name}
								</td>
								<td class="text-xs">
									{#if m.matched}
										S{String(m.seasonNumber).padStart(2, '0')}E{String(m.episodeNumber).padStart(
											2,
											'0'
										)}
									{:else}
										—
									{/if}
								</td>
								<td class="max-w-xs truncate text-xs opacity-70">{m.episodeName}</td>
								<td>
									{#if m.matched}
										<span class="badge badge-xs badge-success">matched</span>
									{:else}
										<span class="badge badge-xs badge-warning">skip</span>
									{/if}
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>

			<div class="mt-3 flex items-center justify-between">
				<span class="text-xs opacity-50">
					{matched.length} matched · {unmatched.length} skipped
				</span>
				<div class="flex gap-2">
					<button class="btn btn-ghost btn-sm" onclick={onclose}>Cancel</button>
					<button class="btn btn-sm btn-primary" onclick={confirm} disabled={matched.length === 0}>
						Link {matched.length} file{matched.length !== 1 ? 's' : ''}
					</button>
				</div>
			</div>
		{/if}
	</div>
	<div class="modal-backdrop" onclick={onclose}></div>
</div>
