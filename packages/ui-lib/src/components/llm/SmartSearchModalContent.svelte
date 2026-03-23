<script lang="ts">
	import { onMount } from 'svelte';
	import classNames from 'classnames';
	import { smartSearchService } from 'ui-lib/services/smart-search.service';
	import { RA_CONSOLES } from 'addons/retroachievements/types';
	import type { SmartSearchMediaType } from 'ui-lib/types/smart-search.type';

	const configStore = smartSearchService.configStore;

	let activeTab = $state<SmartSearchMediaType>('movies');

	const languages = [
		'English',
		'Spanish',
		'French',
		'German',
		'Italian',
		'Portuguese',
		'Russian',
		'Japanese',
		'Korean',
		'Chinese',
		'Hindi',
		'Arabic',
		'Dutch',
		'Swedish',
		'Norwegian',
		'Danish',
		'Finnish',
		'Polish',
		'Turkish',
		'Thai'
	];
	const videoQualities = ['4K', '2160p', '1080p', '720p', '480p'];
	const audioQualities = ['FLAC', 'ALAC', 'Lossless', '320kbps', 'MP3', 'AAC', 'WAV', 'OGG'];

	const bookFormats = ['EPUB', 'PDF', 'MOBI', 'AZW3'];

	const tabs: { id: SmartSearchMediaType; label: string }[] = [
		{ id: 'movies', label: 'Movies' },
		{ id: 'tv', label: 'TV' },
		{ id: 'music', label: 'Music' },
		{ id: 'games', label: 'Games' },
		{ id: 'books', label: 'Books' }
	];

	let currentConfig = $derived($configStore[activeTab]);

	onMount(() => {
		smartSearchService.initializeConfig();
	});
</script>

<div class="flex flex-col gap-6 p-4">
	<div class="flex items-center justify-between">
		<h2 class="text-lg font-bold">Smart Search</h2>
		<div role="tablist" class="tabs-boxed tabs tabs-sm">
			{#each tabs as tab}
				<button
					role="tab"
					class={classNames('tab', { 'tab-active': activeTab === tab.id })}
					onclick={() => (activeTab = tab.id)}
				>
					{tab.label}
				</button>
			{/each}
		</div>
	</div>

	<div class="space-y-4">
		<h3 class="text-sm font-semibold">Preferences</h3>
		<div class="flex flex-wrap items-center gap-4">
			{#if activeTab === 'movies' || activeTab === 'tv'}
				<label class="flex items-center gap-2 text-sm">
					<span class="text-base-content/60">Language</span>
					<select
						class="select-bordered select select-sm"
						value={currentConfig.preferredLanguage ?? 'English'}
						onchange={(e) =>
							smartSearchService.updateConfig(
								activeTab,
								'preferredLanguage',
								e.currentTarget.value
							)}
					>
						{#each languages as lang}
							<option value={lang}>{lang}</option>
						{/each}
					</select>
				</label>
				<label class="flex items-center gap-2 text-sm">
					<span class="text-base-content/60">Quality</span>
					<select
						class="select-bordered select select-sm"
						value={currentConfig.preferredQuality ?? '1080p'}
						onchange={(e) =>
							smartSearchService.updateConfig(activeTab, 'preferredQuality', e.currentTarget.value)}
					>
						{#each videoQualities as q}
							<option value={q}>{q}</option>
						{/each}
					</select>
				</label>
			{:else if activeTab === 'music'}
				<label class="flex items-center gap-2 text-sm">
					<span class="text-base-content/60">Quality</span>
					<select
						class="select-bordered select select-sm"
						value={currentConfig.preferredQuality ?? 'FLAC'}
						onchange={(e) =>
							smartSearchService.updateConfig(activeTab, 'preferredQuality', e.currentTarget.value)}
					>
						{#each audioQualities as q}
							<option value={q}>{q}</option>
						{/each}
					</select>
				</label>
			{:else if activeTab === 'games'}
				<label class="flex items-center gap-2 text-sm">
					<span class="text-base-content/60">Console</span>
					<select
						class="select-bordered select select-sm"
						value={currentConfig.preferredConsole ?? ''}
						onchange={(e) =>
							smartSearchService.updateConfig(activeTab, 'preferredConsole', e.currentTarget.value)}
					>
						<option value="">Any</option>
						{#each RA_CONSOLES as console}
							<option value={console.name}>{console.name}</option>
						{/each}
					</select>
				</label>
			{:else if activeTab === 'books'}
				<label class="flex items-center gap-2 text-sm">
					<span class="text-base-content/60">Format</span>
					<select
						class="select-bordered select select-sm"
						value={currentConfig.preferredFormat ?? 'EPUB'}
						onchange={(e) =>
							smartSearchService.updateConfig(activeTab, 'preferredFormat', e.currentTarget.value)}
					>
						{#each bookFormats as fmt}
							<option value={fmt}>{fmt}</option>
						{/each}
					</select>
				</label>
			{/if}
		</div>
	</div>

	<div class="space-y-2">
		<h3 class="text-sm font-semibold">LLM Prompt</h3>
		<p class="text-xs text-base-content/50">
			Template used when the LLM analyzes torrent results for
			<span class="font-medium">{tabs.find((t) => t.id === activeTab)?.label}</span>.
		</p>
		<textarea
			class="textarea-bordered textarea w-full font-mono text-xs leading-relaxed"
			rows="8"
			value={currentConfig.smartSearchPrompt}
			onchange={(e) =>
				smartSearchService.updateConfig(activeTab, 'smartSearchPrompt', e.currentTarget.value)}
		></textarea>
	</div>
</div>
