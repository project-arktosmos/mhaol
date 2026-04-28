<script lang="ts">
	import classNames from 'classnames';
	import Modal from 'ui-lib/components/core/Modal.svelte';
	import { subtitlesService } from 'ui-lib/services/subtitles.service';
	import type { SubtitleSearchContext } from 'ui-lib/types/subtitles.type';

	let {
		open = false,
		context,
		onclose
	}: {
		open?: boolean;
		context: SubtitleSearchContext | null;
		onclose?: () => void;
	} = $props();

	const subsState = subtitlesService.state;

	let languageFilter = $state('');
	let hearingImpaired = $state(false);
	let userEditedFilter = $state(false);

	// Sync the input with the auto-detected defaults until the user edits it manually.
	$effect(() => {
		if (userEditedFilter) return;
		const next = $subsState.lastLanguages.join(',');
		if (languageFilter !== next) languageFilter = next;
	});

	$effect(() => {
		if (open) {
			subtitlesService.setContext(context);
		}
	});

	function handleSearch() {
		userEditedFilter = true;
		const langs = languageFilter
			.split(',')
			.map((s) => s.trim().toLowerCase())
			.filter(Boolean);
		subtitlesService.search(langs.length ? langs : undefined, hearingImpaired || undefined);
	}

	function handleSearchAll() {
		userEditedFilter = true;
		languageFilter = '';
		subtitlesService.search(undefined, hearingImpaired || undefined);
	}

	function handleDownload(id: string) {
		const result = $subsState.results.find((r) => r.id === id);
		if (result) subtitlesService.download(result);
	}

	function handleRemove(id: string) {
		subtitlesService.remove(id);
	}
</script>

<Modal {open} maxWidth="max-w-3xl" {onclose}>
	<div class="flex flex-col gap-3">
		<div class="flex items-center justify-between">
			<h3 class="text-lg font-semibold">Subtitles</h3>
			<button class="btn btn-circle btn-ghost btn-sm" aria-label="Close" onclick={onclose}>
				&times;
			</button>
		</div>

		{#if !context}
			<p class="text-sm opacity-60">No media context to search for.</p>
		{:else}
			<div class="flex flex-wrap items-end gap-2">
				<label class="form-control flex-1">
					<span class="label-text mb-1 text-xs opacity-70"
						>Languages (auto-detected from title)</span
					>
					<input
						type="text"
						class="input-bordered input input-sm"
						placeholder="eng,spa,fre"
						bind:value={languageFilter}
						oninput={() => (userEditedFilter = true)}
					/>
				</label>
				<label class="label cursor-pointer gap-2">
					<input type="checkbox" class="checkbox checkbox-sm" bind:checked={hearingImpaired} />
					<span class="label-text text-xs">Hearing impaired</span>
				</label>
				<button
					class="btn btn-sm btn-primary"
					onclick={handleSearch}
					disabled={$subsState.searching}
				>
					{$subsState.searching ? 'Searching...' : 'Search'}
				</button>
				<button
					class="btn btn-ghost btn-sm"
					onclick={handleSearchAll}
					disabled={$subsState.searching}
				>
					All languages
				</button>
			</div>

			{#if $subsState.error}
				<div class="alert flex flex-col items-start gap-1 py-2 text-xs alert-error">
					<span>{$subsState.error}</span>
					{#if /401|api key/i.test($subsState.error)}
						<span class="opacity-90">
							Wyzie now requires a free API key. Claim one at
							<a
								class="link"
								href="https://sub.wyzie.io/redeem"
								target="_blank"
								rel="noreferrer noopener">sub.wyzie.io/redeem</a
							>
							and save it under <code>wyzie-subs.apiKey</code> in addon settings (or set
							<code>WYZIE_API_KEY</code> before starting the node).
						</span>
					{/if}
				</div>
			{/if}

			{#if $subsState.assigned.length > 0}
				<section>
					<h4 class="text-xs font-semibold tracking-wide uppercase opacity-60">Assigned</h4>
					<ul class="mt-1 flex flex-col gap-1">
						{#each $subsState.assigned as sub}
							<li class="flex items-center justify-between rounded bg-base-200 px-2 py-1 text-sm">
								<div class="flex flex-col">
									<span class="font-medium">{sub.languageName}</span>
									<span class="text-xs opacity-60"
										>{sub.source}{sub.hearingImpaired ? ' · HI' : ''}</span
									>
								</div>
								<button
									class="btn text-error btn-ghost btn-xs"
									onclick={() => handleRemove(sub.id)}
								>
									Remove
								</button>
							</li>
						{/each}
					</ul>
				</section>
			{/if}

			<section>
				<h4 class="text-xs font-semibold tracking-wide uppercase opacity-60">Search results</h4>
				{#if $subsState.results.length === 0 && !$subsState.searching}
					<p class="mt-1 text-sm opacity-60">No results yet — run a search.</p>
				{:else}
					<ul class="mt-1 flex max-h-96 flex-col gap-1 overflow-y-auto">
						{#each $subsState.results as r (r.id)}
							{@const isAssigned = $subsState.assigned.some((a) => a.sourceId === r.id)}
							<li class="flex items-center justify-between rounded bg-base-200 px-2 py-1 text-sm">
								<div class="flex flex-col">
									<span class="font-medium">
										{r.display || r.language}
										{#if r.isHearingImpaired}
											<span class="badge badge-ghost badge-xs">HI</span>
										{/if}
									</span>
									<span class="text-xs opacity-60">
										{r.source} · {r.format}{r.media ? ` · ${r.media}` : ''}
									</span>
								</div>
								<button
									class={classNames('btn btn-xs', isAssigned ? 'btn-ghost' : 'btn-primary')}
									disabled={isAssigned || $subsState.downloading === r.id}
									onclick={() => handleDownload(r.id)}
								>
									{#if isAssigned}
										Added
									{:else if $subsState.downloading === r.id}
										Downloading...
									{:else}
										Add
									{/if}
								</button>
							</li>
						{/each}
					</ul>
				{/if}
			</section>
		{/if}
	</div>
</Modal>
