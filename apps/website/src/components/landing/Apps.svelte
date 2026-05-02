<script lang="ts">
	import { Icon, type IconName } from 'cloud-ui';

	type App = {
		icon: IconName;
		name: string;
		tag: string;
		body: string;
		host: string;
	};

	const apps: App[] = [
		{
			icon: 'delapouite/laptop',
			name: 'Mhaol Cloud',
			tag: 'Tray-only desktop',
			body: 'Single self-contained .app / .msi / .deb / .AppImage. Lives in the system tray; the Open menu launches the WebUI in your default browser.',
			host: 'macOS · Linux · Windows'
		},
		{
			icon: 'delapouite/server-rack',
			name: 'Mhaol Headless',
			tag: 'Terminal-only',
			body: 'Same backend stack, no Tauri, no tray, no window. The right binary for servers, CI hosts, and any machine that should never open a window.',
			host: 'Linux · macOS · Windows'
		},
		{
			icon: 'delapouite/tv',
			name: 'Mhaol Android TV',
			tag: 'Full-screen WebView',
			body: 'A 10-foot remote-friendly viewport into a remote Mhaol cloud. No backend embedded — point it at any Mhaol cloud over your LAN.',
			host: 'Android TV'
		},
		{
			icon: 'delapouite/smartphone',
			name: 'Mhaol Mobile',
			tag: 'Phone + tablet',
			body: 'Full Mhaol experience in your pocket. Embeds the backend so the device is itself a peer in the swarm and can serve the SPA offline.',
			host: 'Android'
		}
	];
</script>

<section id="apps" class="border-b border-base-300 bg-base-200">
	<div class="mx-auto w-full max-w-6xl px-4 py-20">
		<div class="mb-10 max-w-2xl">
			<h2 class="text-3xl font-bold tracking-tight md:text-4xl">One backend, four shells</h2>
			<p class="mt-3 text-base-content/70">
				The same Rust backend and Svelte SPA, packaged for the device you're on.
			</p>
		</div>

		<div class="grid gap-5 md:grid-cols-2">
			{#each apps as app (app.name)}
				<div class="rounded-box border border-base-300 bg-base-100 p-6">
					<div class="flex items-start gap-4">
						<div
							class="flex h-12 w-12 shrink-0 items-center justify-center rounded-md bg-primary/10 text-primary"
						>
							<Icon name={app.icon} size={26} />
						</div>
						<div class="min-w-0 flex-1">
							<div class="flex flex-wrap items-baseline gap-x-3 gap-y-1">
								<h3 class="text-xl font-semibold">{app.name}</h3>
								<span class="badge badge-soft text-xs badge-primary">{app.tag}</span>
							</div>
							<p class="mt-2 text-sm text-base-content/70">{app.body}</p>
							<div class="mt-3 text-xs text-base-content/60">{app.host}</div>
						</div>
					</div>
				</div>
			{/each}
		</div>
	</div>
</section>
