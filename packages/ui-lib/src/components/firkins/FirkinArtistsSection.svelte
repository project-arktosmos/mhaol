<script lang="ts">
	import type { FirkinArtist } from 'ui-lib/types/firkin.type';

	interface Props {
		artists: FirkinArtist[];
		loading?: boolean;
		error?: string | null;
		emptyLabel?: string;
		title?: string;
		/**
		 * When provided, artist cards become links. Receives the artist's
		 * persisted CID (the `id` field on a resolved artist) and returns
		 * the href to navigate to. Cards for transient/un-persisted artists
		 * (no `id`) stay rendered as plain `<div>`s. Lives outside the
		 * component so the host app — which owns the route shape and
		 * `$app/paths` base — drives the URL.
		 */
		artistHref?: (artistId: string) => string;
	}

	let {
		artists,
		loading = false,
		error = null,
		emptyLabel = 'No people or groups attached.',
		title = 'Artists & credits',
		artistHref
	}: Props = $props();

	function initials(name: string): string {
		return name
			.split(/\s+/)
			.filter((p) => p.length > 0)
			.map((p) => p[0]!.toUpperCase())
			.slice(0, 2)
			.join('');
	}

	/**
	 * Roles for display: the resolved multi-role array when present,
	 * otherwise fall back to the single-`role` field used on the inbound
	 * (un-persisted) side. Filters out blanks.
	 */
	function rolesFor(artist: FirkinArtist): string[] {
		const list = artist.roles ?? (artist.role ? [artist.role] : []);
		return list.map((r) => r.trim()).filter((r) => r.length > 0);
	}
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
		<ul class="grid grid-cols-1 gap-2 sm:grid-cols-2">
			{#each artists as artist, i (artist.id ?? `${i}-${artist.name}`)}
				{@const href = artistHref && artist.id ? artistHref(artist.id) : null}
				{@const roles = rolesFor(artist)}
				<li>
					{#if href}
						<a
							{href}
							class="flex items-start gap-3 rounded border border-base-content/10 bg-base-100 p-2 hover:border-base-content/30"
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
							<div class="flex min-w-0 flex-1 flex-col gap-1">
								<span class="truncate text-sm font-medium">{artist.name}</span>
								{#if roles.length > 0}
									<div class="flex flex-wrap gap-1">
										{#each roles as role (role)}
											<span class="badge badge-ghost badge-sm text-[10px]">{role}</span>
										{/each}
									</div>
								{/if}
							</div>
						</a>
					{:else}
						<div
							class="flex items-start gap-3 rounded border border-base-content/10 bg-base-100 p-2"
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
							<div class="flex min-w-0 flex-1 flex-col gap-1">
								<span class="truncate text-sm font-medium">{artist.name}</span>
								{#if roles.length > 0}
									<div class="flex flex-wrap gap-1">
										{#each roles as role (role)}
											<span class="badge badge-ghost badge-sm text-[10px]">{role}</span>
										{/each}
									</div>
								{/if}
							</div>
						</div>
					{/if}
				</li>
			{/each}
		</ul>
	{/if}
</div>
