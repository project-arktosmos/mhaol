<script lang="ts">
	import { getContext } from 'svelte';
	import { MEDIA_BAR_KEY, type MediaBarContext } from 'ui-lib/types/media-bar.type';
	import WebSurfxSearchPage from 'ui-lib/components/websurfx/WebSurfxSearchPage.svelte';

	const mediaBar = getContext<MediaBarContext>(MEDIA_BAR_KEY);
	mediaBar.configure({ title: 'Web Search' });
</script>

<WebSurfxSearchPage />
