<script lang="ts">
	import type { CatalogAuthor, AuthorRole } from 'ui-lib/types/catalog.type';
	import { authorsByRole, formatAuthors } from 'ui-lib/types/catalog.type';

	interface Props {
		authors: CatalogAuthor[];
		role?: AuthorRole;
		layout: 'grid' | 'badges' | 'inline' | 'labeled';
		label?: string;
		maxItems?: number;
	}

	let { authors, role, layout, label, maxItems }: Props = $props();

	let filtered = $derived(role ? authorsByRole(authors, role) : authors);
	let visible = $derived(maxItems ? filtered.slice(0, maxItems) : filtered);
</script>

{#if visible.length > 0}
	{#if layout === 'grid'}
		<div>
			{#if label}
				<h3 class="mb-1 text-xs font-semibold tracking-wide uppercase opacity-50">{label}</h3>
			{/if}
			<div class="grid grid-cols-2 gap-1 text-sm">
				{#each visible as author}
					<div class="flex items-center gap-2">
						{#if author.imageUrl}
							<img
								src={author.imageUrl}
								alt={author.name}
								class="h-8 w-8 rounded-full object-cover"
								loading="lazy"
							/>
						{:else}
							<div
								class="flex h-8 w-8 items-center justify-center rounded-full bg-base-300 text-xs"
							>
								{author.name[0]}
							</div>
						{/if}
						<div>
							<p class="text-xs font-medium">{author.name}</p>
							{#if author.character}
								<p class="text-xs opacity-50">{author.character}</p>
							{/if}
						</div>
					</div>
				{/each}
			</div>
		</div>
	{:else if layout === 'badges'}
		<div>
			{#if label}
				<h3 class="mb-1 text-xs font-semibold tracking-wide uppercase opacity-50">{label}</h3>
			{/if}
			<div class="flex flex-wrap gap-1">
				{#each visible as author}
					<span class="badge badge-ghost">{author.name}</span>
				{/each}
			</div>
		</div>
	{:else if layout === 'inline'}
		<div class="text-sm">
			{#if label}
				<span class="opacity-50">{label}:</span>
			{/if}
			<span class="font-medium">{formatAuthors(visible)}</span>
		</div>
	{:else if layout === 'labeled'}
		{#each visible as author}
			<div>
				<span class="opacity-50">{label ?? author.role}:</span>
				<span class="font-medium">{author.name}</span>
			</div>
		{/each}
	{/if}
{/if}
