<script lang="ts">
	import classNames from 'classnames';
	import type { FirkinArtist } from 'ui-lib/types/firkin.type';

	interface Props {
		artists: FirkinArtist[];
		loading?: boolean;
		error?: string | null;
		emptyLabel?: string;
		title?: string;
	}

	let {
		artists,
		loading = false,
		error = null,
		emptyLabel = 'No people or groups attached.',
		title = 'Artists & credits'
	}: Props = $props();

	function initials(name: string): string {
		return name
			.split(/\s+/)
			.filter((p) => p.length > 0)
			.map((p) => p[0]!.toUpperCase())
			.slice(0, 2)
			.join('');
	}

	type Group = { role: string; people: FirkinArtist[] };
	const grouped = $derived.by<Group[]>(() => {
		const map = new Map<string, FirkinArtist[]>();
		for (const a of artists) {
			const role = (a.role ?? '').trim() || 'Other';
			const arr = map.get(role) ?? [];
			arr.push(a);
			map.set(role, arr);
		}
		const order = [
			'Director',
			'Writer',
			'Screenplay',
			'Story',
			'Producer',
			'Executive Producer',
			'Original Music Composer',
			'Director of Photography',
			'Actor',
			'Channel',
			'Author',
			'Artist',
			'Composer',
			'Lead vocals',
			'Developer',
			'Publisher',
			'Genre',
			'Other'
		];
		const rank = (r: string) => {
			const i = order.indexOf(r);
			return i === -1 ? order.length : i;
		};
		return [...map.entries()]
			.sort(([a], [b]) => {
				const ra = rank(a);
				const rb = rank(b);
				if (ra !== rb) return ra - rb;
				return a.localeCompare(b);
			})
			.map(([role, people]) => ({ role, people }));
	});
</script>

<div class="card border border-base-content/10 bg-base-200 p-4">
	<h2 class="mb-2 text-sm font-semibold text-base-content/70 uppercase">
		{title}{artists.length > 0 ? ` (${artists.length})` : ''}
	</h2>
	{#if loading && artists.length === 0}
		<p class="text-sm text-base-content/60">Loading…</p>
	{:else if error}
		<p class="text-sm text-error">{error}</p>
	{:else if artists.length === 0}
		<p class="text-sm text-base-content/60">{emptyLabel}</p>
	{:else}
		<div class="flex flex-col gap-3">
			{#each grouped as group (group.role)}
				<div class="flex flex-col gap-2">
					<h3 class="text-[10px] font-semibold tracking-wide text-base-content/50 uppercase">
						{group.role}
					</h3>
					<ul class="grid grid-cols-1 gap-2 sm:grid-cols-2">
						{#each group.people as artist, i (artist.id ?? `${group.role}-${i}-${artist.name}`)}
							{@const card_classes = classNames(
								'flex items-start gap-3 rounded border border-base-content/10 bg-base-100 p-2',
								{ 'cursor-pointer hover:border-base-content/30': !!artist.url }
							)}
							{#if artist.url}
								<li>
									<a
										class={card_classes}
										href={artist.url}
										target="_blank"
										rel="noopener noreferrer"
									>
										{#if artist.imageUrl}
											<img
												src={artist.imageUrl}
												alt={artist.name}
												class="h-12 w-12 shrink-0 rounded-full object-cover"
												loading="lazy"
											/>
										{:else}
											<span
												class="flex h-12 w-12 shrink-0 items-center justify-center rounded-full bg-base-300 text-xs font-semibold text-base-content/60"
											>
												{initials(artist.name)}
											</span>
										{/if}
										<div class="flex min-w-0 flex-1 flex-col">
											<span class="truncate text-sm font-medium">{artist.name}</span>
											{#if artist.description}
												<span class="line-clamp-2 text-xs text-base-content/60"
													>{artist.description}</span
												>
											{/if}
											{#if artist.type}
												<span class="text-[10px] text-base-content/50">{artist.type}</span>
											{/if}
										</div>
									</a>
								</li>
							{:else}
								<li class={card_classes}>
									{#if artist.imageUrl}
										<img
											src={artist.imageUrl}
											alt={artist.name}
											class="h-12 w-12 shrink-0 rounded-full object-cover"
											loading="lazy"
										/>
									{:else}
										<span
											class="flex h-12 w-12 shrink-0 items-center justify-center rounded-full bg-base-300 text-xs font-semibold text-base-content/60"
										>
											{initials(artist.name)}
										</span>
									{/if}
									<div class="flex min-w-0 flex-1 flex-col">
										<span class="truncate text-sm font-medium">{artist.name}</span>
										{#if artist.description}
											<span class="line-clamp-2 text-xs text-base-content/60"
												>{artist.description}</span
											>
										{/if}
										{#if artist.type}
											<span class="text-[10px] text-base-content/50">{artist.type}</span>
										{/if}
									</div>
								</li>
							{/if}
						{/each}
					</ul>
				</div>
			{/each}
		</div>
	{/if}
</div>
