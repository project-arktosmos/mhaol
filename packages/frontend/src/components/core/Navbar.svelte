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
		<!-- Desktop: horizontal menu -->
		<ul class="menu menu-horizontal gap-1 px-1 hidden lg:flex">
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

		<!-- Mobile: burger menu -->
		<div class="dropdown dropdown-end lg:hidden">
			<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
			<div tabindex="0" role="button" class="btn btn-ghost">
				<svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="h-6 w-6">
					<path stroke-linecap="round" stroke-linejoin="round" d="M3.75 6.75h16.5M3.75 12h16.5m-16.5 5.25h16.5" />
				</svg>
			</div>
			<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
			<ul tabindex="0" class="menu dropdown-content z-50 mt-3 w-52 rounded-box bg-base-200 p-2 shadow">
				{#each navRoutes as route (route.path)}
					{#if route.path !== '/'}
						{#if hasNavigableChildren(route)}
							<li class="menu-title">{route.label}</li>
							{#each route.children.filter((c) => !c.isDynamic) as child (child.path)}
								<li>
									<NavbarLink href={child.path} label={child.label} />
								</li>
							{/each}
						{:else}
							<li>
								<NavbarLink href={route.path} label={route.label} />
							</li>
						{/if}
					{/if}
				{/each}
			</ul>
		</div>
	</div>
</nav>
