<script lang="ts">
	interface Props {
		versionHashes: string[];
		hrefFor?: ((cid: string) => string) | null;
	}
	let { versionHashes, hrefFor = null }: Props = $props();
</script>

{#if versionHashes.length > 0}
	<div class="card border border-base-content/10 bg-base-200 p-4">
		<h2 class="mb-2 text-sm font-semibold text-base-content/70 uppercase">
			Version history ({versionHashes.length})
		</h2>
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
	</div>
{/if}
