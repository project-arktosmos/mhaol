<script lang="ts">
	const steps = [
		'Pick a title from your library — the metadata (title, year, artist) is read automatically',
		'Torrents are searched using the parsed metadata as the query',
		'Each result is scored against the exact title and your preferences',
		'Results are ranked — the best candidate is auto-selected for you'
	];
</script>

<section id="smart-search" class="py-20 lg:py-28">
	<div class="mx-auto max-w-6xl px-6">
		<div class="grid items-center gap-12 lg:grid-cols-2">
			<div>
				<span class="mb-4 badge badge-lg badge-primary">Automatic</span>
				<h2 class="text-3xl font-bold sm:text-4xl">Smart Search</h2>
				<p class="mt-4 text-base-content/60">
					Select any movie, show, or album from your library and Smart Search does the rest. It
					reads the metadata — title, year, artist — searches for torrents, and ranks each result by
					relevance, quality, and language, automatically picking the best match.
				</p>
				<ul class="mt-6 space-y-3">
					{#each steps as step}
						<li class="flex items-start gap-3">
							<svg
								xmlns="http://www.w3.org/2000/svg"
								fill="none"
								viewBox="0 0 24 24"
								stroke-width="2"
								stroke="currentColor"
								class="mt-0.5 h-5 w-5 shrink-0 text-primary"
							>
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									d="M9 12.75 11.25 15 15 9.75M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z"
								/>
							</svg>
							<span class="text-sm text-base-content/70">{step}</span>
						</li>
					{/each}
				</ul>
			</div>
			<div class="rounded-2xl bg-base-200 p-6">
				<div class="mb-4 flex items-center gap-2">
					<div class="h-3 w-3 rounded-full bg-error"></div>
					<div class="h-3 w-3 rounded-full bg-warning"></div>
					<div class="h-3 w-3 rounded-full bg-success"></div>
					<span class="ml-2 text-xs text-base-content/40">Smart Search</span>
				</div>
				<div class="space-y-3">
					<div class="rounded-lg bg-base-300 px-3 py-2">
						<div class="flex items-center justify-between">
							<div>
								<div class="text-sm font-semibold">Dune: Part Two</div>
								<div class="flex items-center gap-1 text-xs text-base-content/50">
									<span>2024</span>
									<span class="badge badge-xs badge-primary">Movie</span>
								</div>
							</div>
							<span class="text-xs text-base-content/40">metadata</span>
						</div>
						<div class="mt-1 font-mono text-xs text-base-content/40">
							Search: Dune Part Two 2024
						</div>
					</div>
					<div class="rounded-lg bg-primary/10 p-3">
						<div class="mb-2 flex items-center gap-2">
							<svg
								xmlns="http://www.w3.org/2000/svg"
								fill="none"
								viewBox="0 0 24 24"
								stroke-width="1.5"
								stroke="currentColor"
								class="h-4 w-4 text-primary"
							>
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									d="M9.813 15.904 9 18.75l-.813-2.846a4.5 4.5 0 0 0-3.09-3.09L2.25 12l2.846-.813a4.5 4.5 0 0 0 3.09-3.09L9 5.25l.813 2.846a4.5 4.5 0 0 0 3.09 3.09L15.75 12l-2.846.813a4.5 4.5 0 0 0-3.09 3.09ZM18.259 8.715 18 9.75l-.259-1.035a3.375 3.375 0 0 0-2.455-2.456L14.25 6l1.036-.259a3.375 3.375 0 0 0 2.455-2.456L18 2.25l.259 1.035a3.375 3.375 0 0 0 2.455 2.456L21.75 6l-1.036.259a3.375 3.375 0 0 0-2.455 2.456ZM16.894 20.567 16.5 21.75l-.394-1.183a2.25 2.25 0 0 0-1.423-1.423L13.5 18.75l1.183-.394a2.25 2.25 0 0 0 1.423-1.423l.394-1.183.394 1.183a2.25 2.25 0 0 0 1.423 1.423l1.183.394-1.183.394a2.25 2.25 0 0 0-1.423 1.423Z"
								/>
							</svg>
							<span class="text-xs font-semibold text-primary">AI Analysis</span>
						</div>
						<div class="space-y-1 text-xs text-base-content/60">
							<div class="flex justify-between">
								<span>Dune.Part.Two.2024.2160p.BluRay</span>
								<span class="font-mono text-success">95%</span>
							</div>
							<div class="flex justify-between">
								<span>Dune.Part.2.2024.1080p.WEB-DL</span>
								<span class="font-mono text-success">88%</span>
							</div>
							<div class="flex justify-between">
								<span>Dune.2021.Complete.Collection</span>
								<span class="font-mono text-warning">42%</span>
							</div>
						</div>
					</div>
					<div class="flex items-center gap-2 text-xs text-base-content/40">
						<svg
							xmlns="http://www.w3.org/2000/svg"
							fill="none"
							viewBox="0 0 24 24"
							stroke-width="1.5"
							stroke="currentColor"
							class="h-4 w-4 text-success"
						>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								d="M9 12.75 11.25 15 15 9.75M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z"
							/>
						</svg>
						<span>Best match auto-selected based on quality, language, and seeders</span>
					</div>
				</div>
			</div>
		</div>
	</div>
</section>
