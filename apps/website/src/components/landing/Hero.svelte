<script lang="ts">
	import { base } from '$app/paths';
	import { Icon } from 'cloud-ui';
</script>

<section
	class="relative overflow-hidden border-b border-base-300 bg-gradient-to-b from-base-100 to-base-200"
>
	<div class="mx-auto flex w-full max-w-6xl flex-col gap-10 px-4 py-20 md:flex-row md:items-center">
		<div class="flex-1">
			<h1 class="text-4xl leading-tight font-bold tracking-tight md:text-6xl">
				Your media library,<br />
				<span class="text-primary">peer-to-peer</span> and<br />
				self-hosted.
			</h1>
			<p class="mt-5 max-w-xl text-base text-base-content/80 md:text-lg">
				Mhaol is a single binary that runs your private media cloud. It pins your library to IPFS,
				groups files into content-addressed records, and streams them to every screen you own — no
				accounts, no third-party servers.
			</p>
			<div class="mt-7 flex flex-wrap items-center gap-3">
				<a href="{base}/#install" class="btn gap-2 btn-lg btn-primary">
					<Icon name="delapouite/cloud-download" size={20} />
					Get Mhaol
				</a>
				<a href="{base}/#how-it-works" class="btn btn-ghost btn-lg">How it works</a>
			</div>
			<div class="mt-6 flex flex-wrap items-center gap-x-5 gap-y-2 text-sm text-base-content/60">
				<span class="flex items-center gap-1">
					<Icon name="delapouite/laptop" size={14} /> macOS · Linux · Windows
				</span>
				<span class="flex items-center gap-1">
					<Icon name="delapouite/server-rack" size={14} /> Headless server
				</span>
				<span class="flex items-center gap-1">
					<Icon name="delapouite/tv" size={14} /> Android TV
				</span>
				<span class="flex items-center gap-1">
					<Icon name="delapouite/smartphone" size={14} /> Android Mobile
				</span>
			</div>
		</div>

		<div class="flex-1">
			<div class="card border border-base-300 bg-base-100 shadow-xl">
				<div class="card-body p-5">
					<div class="flex items-center gap-2 border-b border-base-300 pb-3">
						<span class="h-3 w-3 rounded-full bg-error/70"></span>
						<span class="h-3 w-3 rounded-full bg-warning/70"></span>
						<span class="h-3 w-3 rounded-full bg-success/70"></span>
						<span class="ml-3 font-mono text-xs text-base-content/60">mhaol-cloud</span>
					</div>
					<div class="mt-3 grid grid-cols-3 gap-3 text-sm">
						<div class="rounded-md bg-base-200 p-3">
							<div class="text-xs text-base-content/60">SurrealDB</div>
							<div class="font-mono text-success">healthy</div>
						</div>
						<div class="rounded-md bg-base-200 p-3">
							<div class="text-xs text-base-content/60">IPFS</div>
							<div class="font-mono text-success">42 pins</div>
						</div>
						<div class="rounded-md bg-base-200 p-3">
							<div class="text-xs text-base-content/60">Torrent</div>
							<div class="font-mono text-success">idle</div>
						</div>
					</div>
					<div class="mt-3 grid grid-cols-2 gap-3 text-sm">
						<div class="rounded-md bg-base-200 p-3">
							<div class="text-xs text-base-content/60">libp2p TCP</div>
							<div class="font-mono">:9900</div>
						</div>
						<div class="rounded-md bg-base-200 p-3">
							<div class="text-xs text-base-content/60">libp2p /ws</div>
							<div class="font-mono">:9901</div>
						</div>
					</div>
					<div class="mt-3 rounded-md border border-base-300 p-3 text-sm">
						<div class="flex items-center gap-2">
							<Icon name="delapouite/film-strip" size={16} class="text-primary" />
							<span class="font-medium">Library scan complete</span>
						</div>
						<div class="mt-1 text-xs text-base-content/60">
							218 files · 14 firkins · grouped + pinned
						</div>
					</div>
				</div>
			</div>
		</div>
	</div>
</section>
