<script lang="ts">
	import { Icon, type IconName } from 'cloud-ui';

	type Step = {
		icon: IconName;
		title: string;
		body: string;
	};

	const steps: Step[] = [
		{
			icon: 'delapouite/files',
			title: 'Point Mhaol at a folder',
			body: 'Add a library directory and tag it with the addons that belong there — local-movie, local-tv, local-album, local-book, local-game.'
		},
		{
			icon: 'delapouite/cube',
			title: 'It pins and groups',
			body: 'Mhaol walks the directory, pins each file to the embedded IPFS node, and groups them into firkins — TV by season, music by album, one record per book or game.'
		},
		{
			icon: 'lorc/aerial-signal',
			title: 'It enriches',
			body: 'Bookmark a firkin and the cloud resolves trailers, metadata, lyrics, subtitles, related YouTube content, and torrents — all behind the same content-addressed identity.'
		},
		{
			icon: 'delapouite/wifi-router',
			title: 'It serves every screen',
			body: 'The same SPA renders in your browser, on your TV, and on your phone. mDNS finds peers on your LAN; libp2p connects them across the internet.'
		}
	];
</script>

<section id="how-it-works" class="border-b border-base-300 bg-base-200">
	<div class="mx-auto w-full max-w-6xl px-4 py-20">
		<div class="mb-10 max-w-2xl">
			<h2 class="text-3xl font-bold tracking-tight md:text-4xl">How it works</h2>
			<p class="mt-3 text-base-content/70">
				Four steps from "I have a hard drive of media" to "I have a private streaming service".
			</p>
		</div>

		<ol class="grid gap-5 md:grid-cols-2">
			{#each steps as step, i (step.title)}
				<li class="flex gap-4 rounded-box border border-base-300 bg-base-100 p-5">
					<div class="flex shrink-0 flex-col items-center gap-2">
						<div
							class="flex h-10 w-10 items-center justify-center rounded-full bg-primary text-primary-content"
						>
							<span class="font-mono text-sm font-bold">{i + 1}</span>
						</div>
						<div class="text-primary">
							<Icon name={step.icon} size={24} />
						</div>
					</div>
					<div class="min-w-0 flex-1">
						<h3 class="font-semibold">{step.title}</h3>
						<p class="mt-1 text-sm text-base-content/70">{step.body}</p>
					</div>
				</li>
			{/each}
		</ol>
	</div>
</section>
