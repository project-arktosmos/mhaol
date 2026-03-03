<script lang="ts">
	import classNames from 'classnames';
	import { modalRouterService } from '$services/modal-router.service';

	interface Props {
		classes?: string;
	}

	let { classes = '' }: Props = $props();

	let wrapperClasses = $derived(classNames('navbar bg-base-100 shadow-sm', classes));
</script>

<nav class={wrapperClasses}>
	<div class="flex-1">
		<a href="/" class="btn btn-ghost text-xl">Mhaol</a>
	</div>

	<div class="flex-none">
		<!-- Desktop: horizontal buttons -->
		<div class="hidden gap-1 lg:flex">
			<button class="btn btn-sm btn-info" onclick={() => modalRouterService.openNavbar('youtube')}>
				YouTube
			</button>
			<button class="btn btn-sm btn-info btn-outline" onclick={() => modalRouterService.openNavbar('youtube-search')}>
				YT Search
			</button>
			<button class="btn btn-sm btn-primary" onclick={() => modalRouterService.openNavbar('torrent')}>
				Torrent
			</button>
			<button class="btn btn-sm btn-secondary" onclick={() => modalRouterService.openNavbar('downloads')}>
				Downloads
			</button>
			<button class="btn btn-sm btn-neutral" onclick={() => modalRouterService.openNavbar('libraries')}>
				Libraries
			</button>
			<button class="btn btn-sm btn-ghost" onclick={() => modalRouterService.openNavbar('signaling')}>
				Signaling
			</button>
			<button class="btn btn-sm btn-ghost" onclick={() => modalRouterService.openNavbar('identity')}>
				Identity
			</button>
			<button class="btn btn-sm btn-ghost" onclick={() => modalRouterService.openNavbar('plugins')}>
				Plugins
			</button>
			<button class="btn btn-sm btn-ghost" onclick={() => modalRouterService.openNavbar('addons')}>
				Addons
			</button>
			<button class="btn btn-sm btn-ghost" onclick={() => modalRouterService.openNavbar('settings')}>
				Settings
			</button>
		</div>

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
				<li><button onclick={() => modalRouterService.openNavbar('youtube')}>YouTube</button></li>
				<li><button onclick={() => modalRouterService.openNavbar('youtube-search')}>YT Search</button></li>
				<li><button onclick={() => modalRouterService.openNavbar('torrent')}>Torrent</button></li>
				<li><button onclick={() => modalRouterService.openNavbar('downloads')}>Downloads</button></li>
				<li><button onclick={() => modalRouterService.openNavbar('libraries')}>Libraries</button></li>
				<li><button onclick={() => modalRouterService.openNavbar('signaling')}>Signaling</button></li>
				<li><button onclick={() => modalRouterService.openNavbar('identity')}>Identity</button></li>
				<li><button onclick={() => modalRouterService.openNavbar('plugins')}>Plugins</button></li>
				<li><button onclick={() => modalRouterService.openNavbar('addons')}>Addons</button></li>
				<li><button onclick={() => modalRouterService.openNavbar('settings')}>Settings</button></li>
			</ul>
		</div>
	</div>
</nav>
