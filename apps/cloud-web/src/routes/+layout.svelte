<script lang="ts">
	import '../css/app.css';
	import 'ui-lib/services/i18n';
	import classNames from 'classnames';
	import Navbar from 'ui-lib/components/core/Navbar.svelte';
	import ThemeToggle from 'ui-lib/components/core/ThemeToggle.svelte';
	import ToastOutlet from 'ui-lib/components/core/ToastOutlet.svelte';
	import { themeService } from 'ui-lib/services/theme.service';
	import { onMount } from 'svelte';
	import { base } from '$app/paths';
	import { NAV_ITEMS, type NavItem } from '$lib/generated/nav';

	let { children } = $props();

	const triggerClass = (item: NavItem) =>
		classNames('btn btn-outline btn-sm', { 'btn-disabled': !item.hasOwnPage });

	onMount(() => {
		themeService.initialize('flix');
	});
</script>

<div class="flex h-screen flex-col">
	<Navbar brand={{ label: 'Mhaol', highlight: 'Cloud' }} classes="!bg-base-300">
		{#snippet center()}
			<div class="flex flex-wrap items-center gap-1">
				{#each NAV_ITEMS as item (item.href)}
					{#if item.children.length === 0}
						<a href="{base}{item.href}" class="btn btn-outline btn-sm">{item.label}</a>
					{:else}
						<div class="dropdown-hover dropdown dropdown-bottom">
							{#if item.hasOwnPage}
								<a href="{base}{item.href}" class={triggerClass(item)}>{item.label}</a>
							{:else}
								<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
								<div tabindex="0" role="button" class={triggerClass(item)}>{item.label}</div>
							{/if}
							<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
							<ul
								tabindex="0"
								class="dropdown-content menu z-50 mt-1 min-w-48 rounded-box bg-base-200 p-2 shadow-lg"
							>
								{#each item.children as child (child.href)}
									<li><a href="{base}{child.href}">{child.label}</a></li>
								{/each}
							</ul>
						</div>
					{/if}
				{/each}
			</div>
		{/snippet}
		{#snippet end()}
			<div class="flex items-center gap-1">
				<ThemeToggle />
			</div>
		{/snippet}
	</Navbar>

	<main class="flex min-w-0 flex-1 overflow-y-auto">
		<div class="relative flex min-w-0 flex-1 flex-col">
			{@render children?.()}
		</div>
	</main>
</div>

<ToastOutlet />
