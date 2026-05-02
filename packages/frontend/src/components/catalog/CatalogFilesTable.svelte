<script lang="ts">
	import { CatalogFilesTable as Shared } from 'cloud-ui';
	import type { FileEntry } from '$lib/firkins.service';

	interface Props {
		files: FileEntry[];
		collapsible?: boolean;
		open?: boolean;
		onToggle?: () => void;
	}
	let { files, collapsible = false, open = true, onToggle }: Props = $props();
</script>

<Shared {files} {collapsible} {open} {onToggle} />
