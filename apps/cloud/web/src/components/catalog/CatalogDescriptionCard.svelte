<script lang="ts">
	import { CatalogDescriptionCard as Shared } from 'cloud-ui';

	interface Props {
		description: string;
	}
	let { description }: Props = $props();
</script>

<Shared {description} />
