<script lang="ts">
	import classNames from 'classnames';
	import type { CloudDocument } from 'ui-lib/types/document.type';

	interface Props {
		document: CloudDocument;
		classes?: string;
	}

	let { document, classes = '' }: Props = $props();

	let coverImage = $derived(document.images?.[0] ?? null);

	function formatDate(value: string | null | undefined): string {
		if (!value) return '';
		try {
			return new Date(value).toLocaleString();
		} catch {
			return value;
		}
	}

	function formatBytes(bytes: number | null | undefined): string {
		if (bytes === null || bytes === undefined || bytes <= 0) return '';
		const units = ['B', 'KB', 'MB', 'GB', 'TB'];
		let size = bytes;
		let unit = 0;
		while (size >= 1024 && unit < units.length - 1) {
			size /= 1024;
			unit++;
		}
		return `${size.toFixed(unit === 0 ? 0 : 1)} ${units[unit]}`;
	}

	let rows = $derived.by(() => {
		const out: { key: string; value: string }[] = [
			{ key: 'ID', value: document.id },
			{ key: 'Type', value: document.type },
			{ key: 'Source', value: document.source },
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
			{ key: 'Images', value: String((document.images ?? []).length) },
			{
				key: 'Files',
				value: (document.files ?? [])
					.map((f) => (f.title ? `${f.type}: ${f.title} (${f.value})` : `${f.type}: ${f.value}`))
					.join('\n')
			},
			{ key: 'Created', value: formatDate(document.created_at) },
			{ key: 'Updated', value: formatDate(document.updated_at) }
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
						<td
							class={classNames(
								'text-xs break-words whitespace-pre-wrap',
								row.key === 'ID' && 'font-mono text-base-content/70'
							)}>{row.value}</td
						>
					</tr>
				{/each}
				{#if coverImage}
					<tr>
						<th class="w-1/3 align-top text-xs font-semibold text-base-content/70">Cover image</th>
						<td class="text-xs text-base-content/70">
							<div>{coverImage.url}</div>
							<div class="opacity-60">
								{coverImage.mimeType}
								{#if coverImage.width && coverImage.height}
									· {coverImage.width}×{coverImage.height}
								{/if}
								{#if coverImage.fileSize}
									· {formatBytes(coverImage.fileSize)}
								{/if}
							</div>
						</td>
					</tr>
				{/if}
			</tbody>
		</table>
	</div>
</article>
