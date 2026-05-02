<script lang="ts">
	import { CatalogDescriptionPanel as Shared } from 'cloud-ui';

	interface Props {
		description: string;
	}

	let { description }: Props = $props();
</script>

<Shared {description} />
