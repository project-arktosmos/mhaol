<script lang="ts">
	import classNames from 'classnames';
	import type { Snippet } from 'svelte';
	import { modalRouterService } from 'ui-lib/services/modal-router.service';

	interface NavbarItem {
		id: string;
		label: string;
		classes?: string;
	}

	let {
		brand = { label: 'Mhaol' },
		items = [],
		classes = '',
		center,
		end,
		children
	}: {
		brand?: { label: string; href?: string; highlight?: string };
		items?: NavbarItem[];
		classes?: string;
		center?: Snippet;
		end?: Snippet;
		children?: Snippet;
	} = $props();

	let wrapperClasses = $derived(classNames('navbar bg-base-100 shadow-sm', classes));
</script>

<nav class={wrapperClasses}>
	<div class="navbar-start">
		<a href={brand.href ?? '/'} class="btn text-xl btn-ghost">
			{brand.label}{#if brand.highlight}<span class="text-primary">{brand.highlight}</span>{/if}
		</a>
	</div>

	{#if center}
		<div class="navbar-center">
			{@render center()}
		</div>
	{/if}

	<div class="navbar-end">
		{#if children}
			{@render children()}
		{:else if items.length > 0}
			<!-- Desktop: horizontal buttons -->
			<div class="hidden gap-1 lg:flex">
				{#each items as item}
					<button
						class={classNames('btn btn-sm', item.classes ?? 'btn-ghost')}
						onclick={() => modalRouterService.openNavbar(item.id)}
					>
						{item.label}
					</button>
				{/each}
			</div>

			<!-- Mobile: burger menu -->
			<div class="dropdown dropdown-end lg:hidden">
				<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
				<div tabindex="0" role="button" class="btn btn-ghost">
					<svg
						xmlns="http://www.w3.org/2000/svg"
						fill="none"
						viewBox="0 0 24 24"
						stroke-width="1.5"
						stroke="currentColor"
						class="h-6 w-6"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							d="M3.75 6.75h16.5M3.75 12h16.5m-16.5 5.25h16.5"
						/>
					</svg>
				</div>
				<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
				<ul
					tabindex="0"
					class="dropdown-content menu z-50 mt-3 w-52 rounded-box bg-base-200 p-2 shadow"
				>
					{#each items as item}
						<li>
							<button onclick={() => modalRouterService.openNavbar(item.id)}>
								{item.label}
							</button>
						</li>
					{/each}
				</ul>
			</div>
		{/if}

		{#if end}
			{@render end()}
		{/if}
	</div>
</nav>
