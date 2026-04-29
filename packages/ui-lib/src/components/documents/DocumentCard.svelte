<script lang="ts">
	import classNames from 'classnames';
	import type { CloudDocument } from 'ui-lib/types/document.type';

	interface Props {
		document: CloudDocument;
		classes?: string;
	}

	let { document, classes = '' }: Props = $props();

	let coverImage = $derived(document.images?.[0] ?? null);

	let hasYear = $derived(document.year !== null && document.year !== undefined);

	let rows = $derived.by(() => {
		const out: { key: string; value: string }[] = [
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
	<header
		class="flex items-baseline justify-between gap-3 border-b border-base-content/10 px-4 py-3"
	>
		<span class="text-xs text-base-content/70">{document.type}</span>
		<h3 class="flex-1 text-center text-base font-semibold [overflow-wrap:anywhere]">
			{document.title}
		</h3>
		<span class="text-xs text-base-content/70">{hasYear ? document.year : ''}</span>
	</header>
	{#if coverImage}
		<figure class="bg-base-300">
			<img
				src={coverImage.url}
				alt={document.title}
				width={coverImage.width || undefined}
				height={coverImage.height || undefined}
				class="block h-auto w-full"
				loading="lazy"
			/>
		</figure>
	{/if}
	<div class="card-body p-0">
		<table class="table w-full table-fixed table-sm">
			<tbody>
				{#each rows as row (row.key)}
					<tr>
						<th class="w-1/3 align-top text-xs font-semibold text-base-content/70">{row.key}</th>
						<td
							class="w-2/3 text-xs whitespace-pre-wrap [overflow-wrap:anywhere] [word-break:break-word]"
							>{row.value}</td
						>
					</tr>
				{/each}
			</tbody>
		</table>
	</div>
</article>
