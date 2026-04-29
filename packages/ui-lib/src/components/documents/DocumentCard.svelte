<script lang="ts">
	import classNames from 'classnames';
	import type { CloudDocument, DocumentFile } from 'ui-lib/types/document.type';

	interface Props {
		document: CloudDocument;
		classes?: string;
	}

	let { document, classes = '' }: Props = $props();

	let coverImage = $derived(document.images?.[0] ?? null);

	let hasYear = $derived(document.year !== null && document.year !== undefined);

	let artistsValue = $derived((document.artists ?? []).map((a) => a.name).join(', '));

	let files = $derived(document.files ?? []);

	function fileTooltip(file: DocumentFile): string {
		return file.title ? `${file.title}\n${file.value}` : file.value;
	}
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
	{#if document.description}
		<p
			class="border-b border-base-content/10 px-4 py-3 text-xs whitespace-pre-wrap [overflow-wrap:anywhere] text-base-content/80"
		>
			{document.description}
		</p>
	{/if}
	<div class="card-body p-0">
		<table class="table w-full table-fixed table-sm">
			<tbody>
				<tr>
					<th class="w-1/3 align-top text-xs font-semibold text-base-content/70">Artists</th>
					<td
						class="w-2/3 text-xs whitespace-pre-wrap [overflow-wrap:anywhere] [word-break:break-word]"
						>{artistsValue}</td
					>
				</tr>
				<tr>
					<th class="w-1/3 align-top text-xs font-semibold text-base-content/70">Files</th>
					<td class="w-2/3 align-top text-xs">
						{#if files.length === 0}
							<span class="text-base-content/50">—</span>
						{:else}
							<ul class="flex flex-wrap items-center gap-2">
								{#each files as file, i (i)}
									<li>
										{#if file.type === 'torrent magnet'}
											<a
												href={file.value}
												title={fileTooltip(file)}
												aria-label={file.title ? `Magnet: ${file.title}` : 'Magnet link'}
												class="inline-flex h-6 w-6 items-center justify-center rounded text-base-content/70 hover:bg-base-300 hover:text-base-content"
											>
												<svg
													xmlns="http://www.w3.org/2000/svg"
													viewBox="0 0 24 24"
													fill="none"
													stroke="currentColor"
													stroke-width="2"
													stroke-linecap="round"
													stroke-linejoin="round"
													class="h-4 w-4"
													aria-hidden="true"
												>
													<path d="M6 3v9a6 6 0 0 0 12 0V3" />
													<path d="M6 8h4" />
													<path d="M14 8h4" />
													<path d="M6 3H3" />
													<path d="M21 3h-3" />
												</svg>
											</a>
										{:else}
											<span
												class="break-all [overflow-wrap:anywhere]"
												title={fileTooltip(file)}
											>
												{file.title ? `${file.type}: ${file.title}` : `${file.type}: ${file.value}`}
											</span>
										{/if}
									</li>
								{/each}
							</ul>
						{/if}
					</td>
				</tr>
			</tbody>
		</table>
	</div>
</article>
