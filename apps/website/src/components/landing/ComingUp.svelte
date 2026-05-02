<script lang="ts">
	import { Icon, type IconName } from 'cloud-ui';

	type Upcoming = {
		icon: IconName;
		title: string;
		body: string;
		points: string[];
	};

	const upcoming: Upcoming[] = [
		{
			icon: 'delapouite/bookshelf',
			title: 'Books',
			body: 'A reading library that lives next to the rest of your media — EPUBs pinned to your cloud, scanned out of a folder you point Mhaol at, and synced to whatever device is in your hand.',
			points: [
				'EPUB / PDF library scanning',
				'Reading progress that follows the firkin, not the device',
				'Cover art and metadata via OpenLibrary'
			]
		},
		{
			icon: 'delapouite/gamepad',
			title: 'Games',
			body: 'A ROM and game library with the same content-addressed semantics as the rest of Mhaol — point at a folder, get a clean catalog with cover art, ready to hand off to the emulator of your choice.',
			points: [
				'ROM library scanning by platform',
				'Cover art + metadata for every system',
				'Launch hand-off to your installed emulator'
			]
		}
	];
</script>

<section id="coming-up" class="border-b border-base-300 bg-base-200">
	<div class="mx-auto w-full max-w-6xl px-4 py-20">
		<div class="mb-10 max-w-3xl">
			<div
				class="mb-3 inline-flex items-center gap-1.5 text-xs font-semibold tracking-wide text-primary uppercase"
			>
				<Icon name="delapouite/info" size={14} />
				Roadmap
			</div>
			<h2 class="text-3xl font-bold tracking-tight md:text-4xl">Coming up</h2>
			<p class="mt-3 text-base-content/70">
				Features that aren't shipping today but are on the roadmap. They show up in the catalog with
				a “Soon” marker until they're ready.
			</p>
		</div>

		<div class="grid gap-5 md:grid-cols-2">
			{#each upcoming as item (item.title)}
				<div
					class="relative flex flex-col rounded-box border border-dashed border-base-300 bg-base-100/60 p-6"
					aria-disabled="true"
				>
					<span
						class="absolute top-4 right-4 badge gap-1 text-[0.65rem] font-semibold tracking-wide uppercase badge-warning"
					>
						<Icon name="delapouite/info" size={12} />
						Coming soon
					</span>

					<div class="flex items-center gap-4">
						<div
							class="flex h-12 w-12 items-center justify-center rounded-md bg-primary/10 text-primary opacity-60"
						>
							<Icon name={item.icon} size={26} />
						</div>
						<h3 class="text-xl font-semibold text-base-content/80">{item.title}</h3>
					</div>

					<p class="mt-4 text-sm text-base-content/70">{item.body}</p>

					<ul
						class="mt-4 space-y-1.5 border-t border-dashed border-base-300 pt-4 text-sm text-base-content/70"
					>
						{#each item.points as point (point)}
							<li class="flex items-start gap-2">
								<span class="mt-1 inline-block h-1.5 w-1.5 shrink-0 rounded-full bg-base-content/30"
								></span>
								<span>{point}</span>
							</li>
						{/each}
					</ul>
				</div>
			{/each}
		</div>
	</div>
</section>
