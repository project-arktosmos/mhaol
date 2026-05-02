<script lang="ts">
	interface Props {
		versionHashes: string[];
		hrefFor?: ((cid: string) => string) | null;
	}
	let { versionHashes, hrefFor = null }: Props = $props();
</script>

{#if versionHashes.length > 0}
	<div class="flex flex-col gap-2">
		<h2 class="text-sm font-semibold text-base-content/70 uppercase">
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
