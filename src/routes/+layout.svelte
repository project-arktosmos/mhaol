<script lang="ts">
	import '../css/app.css';
	import '$services/i18n';

	let { children } = $props();
</script>

{@render children?.()}
