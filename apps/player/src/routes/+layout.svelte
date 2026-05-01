<script lang="ts">
	import '../css/app.css';
	import { resolve } from '$app/paths';

	interface Props {
		children?: import('svelte').Snippet;
	}
	let { children }: Props = $props();
</script>

<header class="navbar bg-base-200">
	<div class="navbar-start">
		<a href={resolve('/player')} class="btn text-lg font-bold btn-ghost">Mhaol Player</a>
	</div>
</header>
<main class="min-h-screen bg-base-100">
	{@render children?.()}
</main>
