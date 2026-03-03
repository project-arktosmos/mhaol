<script lang="ts">
	import classNames from 'classnames';
	import { page } from '$app/stores';
	import type { RouteEntry } from '$types/route.type';

	interface Props {
		route: RouteEntry;
		classes?: string;
	}

	let { route, classes = '' }: Props = $props();

	let navigableChildren = $derived(route.children.filter((c) => !c.isDynamic));

	let isActive = $derived(
		$page.url.pathname === route.path || $page.url.pathname.startsWith(route.path + '/')
	);

	let dropdownClasses = $derived(classNames('dropdown dropdown-end', classes));

	let buttonClasses = $derived(
		classNames('btn btn-ghost btn-sm', {
			'btn-active': isActive
		})
	);
</script>

<div class={dropdownClasses}>
	<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
	<div tabindex="0" role="button" class={buttonClasses}>
		<a href={route.path}>{route.label}</a>
		<svg
			xmlns="http://www.w3.org/2000/svg"
			fill="none"
			viewBox="0 0 24 24"
			stroke-width="1.5"
			stroke="currentColor"
			class="h-4 w-4"
		>
			<path stroke-linecap="round" stroke-linejoin="round" d="M19.5 8.25l-7.5 7.5-7.5-7.5" />
		</svg>
	</div>
	<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
	<ul tabindex="0" class="dropdown-content menu z-50 mt-1 w-52 rounded-box bg-base-200 p-2 shadow">
		{#each navigableChildren as child (child.path)}
			<li>
				<a href={child.path} class={classNames({ active: $page.url.pathname === child.path })}>
					{child.label}
				</a>
			</li>
		{/each}
	</ul>
</div>
