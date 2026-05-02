<script lang="ts">
	interface Props {
		cid: string;
		createdAt: string;
		updatedAt: string;
		version: number;
	}
	let { cid, createdAt, updatedAt, version }: Props = $props();

	function formatDate(value: string): string {
		try {
			return new Date(value).toLocaleString();
		} catch {
			return value;
		}
	}
</script>

<div class="card border border-base-content/10 bg-base-200 p-4">
	<h2 class="mb-2 text-sm font-semibold text-base-content/70 uppercase">Identity</h2>
	<table class="table table-sm">
		<tbody>
			<tr>
				<th class="w-32 align-top">CID</th>
				<td class="font-mono text-xs break-all">{cid}</td>
			</tr>
			<tr>
				<th class="w-32 align-top">Created</th>
				<td class="text-xs">{formatDate(createdAt)}</td>
			</tr>
			<tr>
				<th class="w-32 align-top">Updated</th>
				<td class="text-xs">{formatDate(updatedAt)}</td>
			</tr>
			<tr>
				<th class="w-32 align-top">Version</th>
				<td class="text-xs">{version}</td>
			</tr>
		</tbody>
	</table>
</div>
