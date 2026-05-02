<script lang="ts">
	interface Identity {
		cid: string;
		createdAt: string;
		updatedAt: string;
		version: number;
	}

	interface Review {
		label: string;
		score: number;
		maxScore: number;
		voteCount?: number;
	}

	interface Props {
		description: string;
		identity?: Identity | null;
		versionHashes?: string[];
		reviews?: Review[];
		hrefFor?: ((cid: string) => string) | null;
	}

	let {
		description,
		identity = null,
		versionHashes = [],
		reviews = [],
		hrefFor = null
	}: Props = $props();

	type TabId = 'description' | 'identity' | 'versions';

	let activeTab = $state<TabId>('description');

	const tabs = $derived<{ id: TabId; label: string }[]>(
		[
			{ id: 'description' as const, label: 'Description', show: true },
			{ id: 'identity' as const, label: 'Identity', show: !!identity },
			{
				id: 'versions' as const,
				label: `Version history${versionHashes.length > 0 ? ` (${versionHashes.length})` : ''}`,
				show: versionHashes.length > 0
			}
		]
			.filter((t) => t.show)
			.map(({ id, label }) => ({ id, label }))
	);

	function formatDate(value: string): string {
		try {
			return new Date(value).toLocaleString();
		} catch {
			return value;
		}
	}

	function formatVotes(count: number): string {
		if (count >= 1_000_000) return `${(count / 1_000_000).toFixed(1)}M votes`;
		if (count >= 1000) return `${(count / 1000).toFixed(1)}k votes`;
		return `${count} vote${count === 1 ? '' : 's'}`;
	}

	function reviewPercent(review: Review): number | null {
		if (!Number.isFinite(review.maxScore) || review.maxScore <= 0) return null;
		const ratio = review.score / review.maxScore;
		if (!Number.isFinite(ratio)) return null;
		return Math.max(0, Math.min(100, ratio * 100));
	}

	function formatPercent(value: number): string {
		return `${Math.round(value)}%`;
	}

	const reviewPercents = $derived(
		reviews
			.map((r) => reviewPercent(r))
			.filter((v): v is number => typeof v === 'number')
	);
	const averagePercent = $derived(
		reviewPercents.length > 0
			? reviewPercents.reduce((sum, v) => sum + v, 0) / reviewPercents.length
			: null
	);
</script>

<div class="card border border-base-content/10 bg-base-200 p-4">
	{#if reviews.length > 0}
		<div class="mb-3 overflow-x-auto rounded border border-base-content/10">
			<table class="table table-xs">
				<thead>
					<tr>
						<th>Source</th>
						<th class="text-right">Score</th>
						<th class="w-32 text-right">Votes</th>
					</tr>
				</thead>
				<tbody>
					{#each reviews as review (review.label)}
						{@const pct = reviewPercent(review)}
						<tr>
							<td class="font-semibold">{review.label}</td>
							<td class="text-right font-mono">
								{#if pct !== null}
									{formatPercent(pct)}
								{:else}
									—
								{/if}
							</td>
							<td class="text-right text-base-content/60">
								{#if review.voteCount !== undefined}
									{formatVotes(review.voteCount)}
								{:else}
									—
								{/if}
							</td>
						</tr>
					{/each}
					{#if averagePercent !== null && reviewPercents.length > 1}
						<tr class="border-t-2 border-base-content/20 bg-base-300/40 font-semibold">
							<td>Average</td>
							<td class="text-right font-mono">{formatPercent(averagePercent)}</td>
							<td></td>
						</tr>
					{/if}
				</tbody>
			</table>
		</div>
	{/if}

	{#if tabs.length > 1}
		<div role="tablist" class="tabs-boxed tabs tabs-sm mb-3 self-start">
			{#each tabs as tab (tab.id)}
				<button
					type="button"
					role="tab"
					class="tab"
					class:tab-active={activeTab === tab.id}
					onclick={() => (activeTab = tab.id)}
				>
					{tab.label}
				</button>
			{/each}
		</div>
	{:else}
		<h2 class="mb-2 text-sm font-semibold text-base-content/70 uppercase">Description</h2>
	{/if}

	{#if activeTab === 'description'}
		{#if description}
			<p class="text-sm [overflow-wrap:anywhere] whitespace-pre-wrap">{description}</p>
		{:else}
			<p class="text-sm text-base-content/60 italic">No description.</p>
		{/if}
	{:else if activeTab === 'identity' && identity}
		<table class="table table-sm">
			<tbody>
				<tr>
					<th class="w-32 align-top">CID</th>
					<td class="font-mono text-xs break-all">{identity.cid}</td>
				</tr>
				<tr>
					<th class="w-32 align-top">Created</th>
					<td class="text-xs">{formatDate(identity.createdAt)}</td>
				</tr>
				<tr>
					<th class="w-32 align-top">Updated</th>
					<td class="text-xs">{formatDate(identity.updatedAt)}</td>
				</tr>
				<tr>
					<th class="w-32 align-top">Version</th>
					<td class="text-xs">{identity.version}</td>
				</tr>
			</tbody>
		</table>
	{:else if activeTab === 'versions'}
		<ol class="list-decimal pl-6 text-xs">
			{#each versionHashes as cid, i (i)}
				<li class="font-mono break-all">
					{#if hrefFor}
						<a class="link" href={hrefFor(cid)}>{cid}</a>
					{:else}
						<span>{cid}</span>
					{/if}
				</li>
			{/each}
		</ol>
	{/if}
</div>
