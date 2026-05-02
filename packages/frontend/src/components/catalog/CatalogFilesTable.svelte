<script lang="ts">
	import { CatalogFilesTable as Shared } from 'cloud-ui';
	import type { FileEntry } from '$lib/firkins.service';

	interface Props {
		files: FileEntry[];
	}
	let { files }: Props = $props();
</script>

<Shared {files} />
