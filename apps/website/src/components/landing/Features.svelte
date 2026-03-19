<script lang="ts">
	import { _ } from 'svelte-i18n';
	import FeatureCard from './FeatureCard.svelte';

	const features = [
		{
			key: 'downloader',
			icon: 'M3 16.5v2.25A2.25 2.25 0 0 0 5.25 21h13.5A2.25 2.25 0 0 0 21 18.75V16.5M16.5 12 12 16.5m0 0L7.5 12m4.5 4.5V3'
		},
		{
			key: 'privacy',
			icon: 'M9 12.75 11.25 15 15 9.75m-3-7.036A11.959 11.959 0 0 1 3.598 6 11.99 11.99 0 0 0 3 9.749c0 5.592 3.824 10.29 9 11.623 5.176-1.332 9-6.03 9-11.622 0-1.31-.21-2.571-.598-3.751h-.152c-3.196 0-6.1-1.248-8.25-3.285z'
		},
		{
			key: 'nodata',
			icon: 'M16.5 10.5V6.75a4.5 4.5 0 1 0-9 0v3.75m-.75 11.25h10.5a2.25 2.25 0 0 0 2.25-2.25v-6.75a2.25 2.25 0 0 0-2.25-2.25H6.75a2.25 2.25 0 0 0-2.25 2.25v6.75a2.25 2.25 0 0 0 2.25 2.25z'
		},
		{
			key: 'client',
			icon: 'M5.25 5.653c0-.856.917-1.398 1.667-.986l11.54 6.347a1.125 1.125 0 0 1 0 1.972l-11.54 6.347a1.125 1.125 0 0 1-1.667-.986V5.653z'
		},
		{
			key: 'free',
			icon: 'M12 6v12m-3-2.818.879.659c1.171.879 3.07.879 4.242 0 1.172-.879 1.172-2.303 0-3.182C13.536 12.219 12.768 12 12 12c-.725 0-1.45-.22-2.003-.659-1.106-.879-1.106-2.303 0-3.182s2.9-.879 4.006 0l.415.33M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0z'
		},
		{
			key: 'opensource',
			icon: 'M17.25 6.75 22.5 12l-5.25 5.25m-10.5 0L1.5 12l5.25-5.25m7.5-3-4.5 16.5'
		}
	];
</script>

<section id="features" class="bg-base-200 py-20 lg:py-28">
	<div class="mx-auto max-w-6xl px-6">
		<div class="mb-16 text-center">
			<h2 class="text-3xl font-bold sm:text-4xl">{$_('features.heading')}</h2>
			<p class="mt-4 text-base-content/60">{$_('features.subheading')}</p>
		</div>
		<div class="grid gap-6 sm:grid-cols-2 lg:grid-cols-3">
			{#each features as feature}
				<FeatureCard
					title={$_(`features.${feature.key}.title`)}
					description={$_(`features.${feature.key}.description`)}
					icon={feature.icon}
				/>
			{/each}
		</div>
	</div>
</section>
