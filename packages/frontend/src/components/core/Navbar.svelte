<script lang="ts">
	import classNames from 'classnames';
	import routes from 'virtual:routes';
	import type { RouteEntry } from '$types/route.type';
	import { isMobile } from '$lib/platform';
	import NavbarLink from './NavbarLink.svelte';
	import NavbarDropdown from './NavbarDropdown.svelte';

	interface Props {
		classes?: string;
	}

	let { classes = '' }: Props = $props();

	const DESKTOP_ONLY_ROUTES = ['/images', '/signaling'];

	function hasNavigableChildren(route: RouteEntry): boolean {
		return route.children.some((c) => !c.isDynamic);
	}

	let navRoutes = $derived(
		routes.filter(
			(r) => !r.isDynamic && !(isMobile && DESKTOP_ONLY_ROUTES.includes(r.path))
		)
	);

	let wrapperClasses = $derived(classNames('navbar bg-base-100 shadow-sm', classes));
</script>

<nav class={wrapperClasses}>
	<div class="flex-1">
		<a href="/" class="btn btn-ghost text-xl">Mhaol</a>
	</div>

	<div class="flex-none">
		<ul class="menu menu-horizontal gap-1 px-1">
			{#each navRoutes as route (route.path)}
				{#if route.path !== '/'}
					{#if hasNavigableChildren(route)}
						<li>
							<NavbarDropdown {route} />
						</li>
					{:else}
						<li>
							<NavbarLink href={route.path} label={route.label} />
						</li>
					{/if}
				{/if}
			{/each}
		</ul>
	</div>
</nav>
