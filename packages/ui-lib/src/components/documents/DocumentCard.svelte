<script lang="ts">
	import classNames from 'classnames';
	import type { CloudDocument } from 'ui-lib/types/document.type';

	interface Props {
		document: CloudDocument;
		classes?: string;
	}

	let { document, classes = '' }: Props = $props();

	let coverImage = $derived(document.images?.[0] ?? null);

	let rows = $derived.by(() => {
		const out: { key: string; value: string }[] = [
			{ key: 'Type', value: document.type },
			{ key: 'Title', value: document.title },
			{
				key: 'Year',
				value: document.year !== null && document.year !== undefined ? String(document.year) : ''
			},
			{
				key: 'Artists',
				value: (document.artists ?? []).map((a) => a.name).join(', ')
			},
			{ key: 'Description', value: document.description ?? '' },
			{
				key: 'Files',
				value: (document.files ?? [])
					.map((f) => (f.title ? `${f.type}: ${f.title} (${f.value})` : `${f.type}: ${f.value}`))
					.join('\n')
			}
		];
		return out;
	});
</script>

<article class={classNames('card bg-base-200 shadow-sm', classes)}>
	{#if coverImage}
		<figure class="aspect-[3/4] overflow-hidden bg-base-300">
			<img
				src={coverImage.url}
				alt={document.title}
				width={coverImage.width || undefined}
				height={coverImage.height || undefined}
				class="h-full w-full object-cover"
				loading="lazy"
			/>
		</figure>
	{/if}
	<div class="card-body p-0">
		<table class="table w-full table-sm">
			<tbody>
				{#each rows as row (row.key)}
					<tr>
						<th class="w-1/3 align-top text-xs font-semibold text-base-content/70">{row.key}</th>
						<td class="text-xs break-words whitespace-pre-wrap">{row.value}</td>
					</tr>
				{/each}
			</tbody>
		</table>
	</div>
</article>
