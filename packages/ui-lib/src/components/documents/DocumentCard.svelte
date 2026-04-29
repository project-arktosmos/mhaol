<script lang="ts">
	import classNames from 'classnames';
	import type { CloudDocument, DocumentFile } from 'ui-lib/types/document.type';

	interface Props {
		document: CloudDocument;
		classes?: string;
		onRemove?: (id: string) => void;
		removing?: boolean;
	}

	let { document, classes = '', onRemove, removing = false }: Props = $props();

	let coverImage = $derived(document.images?.[0] ?? null);

	let hasYear = $derived(document.year !== null && document.year !== undefined);

	let files = $derived(document.files ?? []);
	let magnetFiles = $derived(files.filter((f) => f.type === 'torrent magnet'));
	let tableFiles = $derived(files.filter((f) => f.type !== 'torrent magnet'));

	function fileTooltip(file: DocumentFile): string {
		return file.title ? `${file.title}\n${file.value}` : file.value;
	}
</script>

<article class={classNames('card group bg-base-200 shadow-sm', classes)}>
	<header
		class="flex items-baseline justify-between gap-3 border-b border-base-content/10 px-4 py-3"
	>
		<span class="text-xs text-base-content/70">{document.type}</span>
		<h3 class="flex-1 text-center text-base font-semibold [overflow-wrap:anywhere]">
			{document.title}
		</h3>
		<span class="text-xs text-base-content/70">{hasYear ? document.year : ''}</span>
		{#if onRemove}
			<button
				type="button"
				class="btn text-error btn-ghost btn-xs"
				onclick={() => onRemove?.(document.id)}
				disabled={removing}
				aria-label="Remove document"
			>
				{removing ? '…' : '×'}
			</button>
		{/if}
	</header>
	{#if coverImage}
		<figure class="relative overflow-hidden bg-base-300">
			<img
				src={coverImage.url}
				alt={document.title}
				width={coverImage.width || undefined}
				height={coverImage.height || undefined}
				class="block h-auto w-full"
				loading="lazy"
			/>
			{#if document.description}
				<figcaption
					class="pointer-events-none absolute inset-x-0 bottom-0 bg-black/50 px-4 py-3 text-xs whitespace-pre-wrap text-white opacity-0 transition-opacity [overflow-wrap:anywhere] group-hover:opacity-100"
				>
					{document.description}
				</figcaption>
			{/if}
		</figure>
	{:else if document.description}
		<p
			class="border-b border-base-content/10 px-4 py-3 text-xs whitespace-pre-wrap [overflow-wrap:anywhere] text-base-content/80"
		>
			{document.description}
		</p>
	{/if}
	{#if tableFiles.length > 0}
		<details class="group border-t border-base-content/10">
			<summary
				class="flex cursor-pointer items-center justify-between px-4 py-2 text-xs font-semibold text-base-content/70 hover:bg-base-300"
			>
				<span>Files ({tableFiles.length})</span>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
					class="h-3 w-3 transition-transform group-open:rotate-180"
					aria-hidden="true"
				>
					<polyline points="6 9 12 15 18 9" />
				</svg>
			</summary>
			<table class="table w-full table-fixed table-sm">
				<tbody>
					{#each tableFiles as file, i (i)}
						<tr>
							<th class="w-1/3 align-top text-xs font-semibold text-base-content/70"
								>{file.type}</th
							>
							<td
								class="w-2/3 text-xs whitespace-pre-wrap [overflow-wrap:anywhere] [word-break:break-word]"
								title={fileTooltip(file)}>{file.title ?? file.value}</td
							>
						</tr>
					{/each}
				</tbody>
			</table>
		</details>
	{/if}
	{#if magnetFiles.length > 0}
		<footer class="flex flex-wrap items-center gap-2 border-t border-base-content/10 px-4 py-3">
			{#each magnetFiles as file, i (i)}
				<a
					href={file.value}
					title={fileTooltip(file)}
					aria-label={file.title ? `Magnet: ${file.title}` : 'Magnet link'}
					class="inline-flex h-8 w-8 items-center justify-center rounded text-base-content/70 hover:bg-base-300 hover:text-base-content"
				>
					<svg
						xmlns="http://www.w3.org/2000/svg"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="2"
						stroke-linecap="round"
						stroke-linejoin="round"
						class="h-5 w-5"
						aria-hidden="true"
					>
						<path d="M6 3v9a6 6 0 0 0 12 0V3" />
						<path d="M6 8h4" />
						<path d="M14 8h4" />
						<path d="M6 3H3" />
						<path d="M21 3h-3" />
					</svg>
				</a>
			{/each}
		</footer>
	{/if}
</article>
