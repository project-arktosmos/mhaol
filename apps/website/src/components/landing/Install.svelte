<script lang="ts">
	import { Icon, type IconName } from 'cloud-ui';

	type Build = {
		icon: IconName;
		title: string;
		command: string;
		artifact: string;
	};

	const builds: Build[] = [
		{
			icon: 'delapouite/laptop',
			title: 'Desktop tray (Mhaol Cloud)',
			command: 'pnpm build:dist',
			artifact: '.app · .msi · .deb · .AppImage'
		},
		{
			icon: 'delapouite/server-rack',
			title: 'Headless server',
			command: 'pnpm build:headless',
			artifact: 'mhaol-headless binary'
		},
		{
			icon: 'delapouite/tv',
			title: 'Android TV',
			command: 'pnpm app:tauri:android:tv:build',
			artifact: 'APK · AAB'
		},
		{
			icon: 'delapouite/smartphone',
			title: 'Android Mobile',
			command: 'pnpm app:tauri:android:mobile:build',
			artifact: 'APK · AAB'
		}
	];
</script>

<section id="install" class="bg-base-100">
	<div class="mx-auto w-full max-w-6xl px-4 py-20">
		<div class="mb-10 max-w-2xl">
			<h2 class="text-3xl font-bold tracking-tight md:text-4xl">Install</h2>
			<p class="mt-3 text-base-content/70">
				Mhaol is a monorepo that produces every shell from one source tree. Pick the binary that
				fits your host.
			</p>
		</div>

		<div class="grid gap-5 md:grid-cols-2">
			{#each builds as build (build.title)}
				<div class="rounded-box border border-base-300 bg-base-100 p-5">
					<div class="flex items-center gap-3">
						<div
							class="flex h-10 w-10 items-center justify-center rounded-md bg-primary/10 text-primary"
						>
							<Icon name={build.icon} size={22} />
						</div>
						<div class="min-w-0 flex-1">
							<div class="font-semibold">{build.title}</div>
							<div class="text-xs text-base-content/60">{build.artifact}</div>
						</div>
					</div>
					<pre class="mt-4 overflow-x-auto rounded-md bg-base-200 p-3 font-mono text-xs"><code
							>{build.command}</code
						></pre>
				</div>
			{/each}
		</div>

		<div class="mt-10 rounded-box border border-base-300 bg-base-200 p-5">
			<div class="font-semibold">First time? Bootstrap your host:</div>
			<div class="mt-3 grid gap-3 md:grid-cols-3">
				<pre class="overflow-x-auto rounded-md bg-base-100 p-3 font-mono text-xs"><code
						>pnpm setup:mac</code
					></pre>
				<pre class="overflow-x-auto rounded-md bg-base-100 p-3 font-mono text-xs"><code
						>pnpm setup:linux</code
					></pre>
				<pre class="overflow-x-auto rounded-md bg-base-100 p-3 font-mono text-xs"><code
						>pnpm setup:windows</code
					></pre>
			</div>
			<p class="mt-3 text-sm text-base-content/70">
				These scripts are idempotent — they install Rust, Node, pnpm, the Tauri CLI, and the
				platform GStreamer + WebKit deps for your OS.
			</p>
		</div>
	</div>
</section>
