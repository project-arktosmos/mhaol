<script lang="ts">
	import { assets } from '$app/paths';
	import { _ } from 'svelte-i18n';
</script>

<section
	class="hero bg-cover bg-center bg-no-repeat"
	style:background-image="url('{assets}/pexels-enginakyurt-1493226.jpg')"
>
	<div class="hero-overlay bg-black/50"></div>
	<div class="hero-content flex-col py-[100px] text-center text-white">
		<div class="max-w-2xl">
			<h1 class="text-5xl font-bold sm:text-6xl lg:text-7xl">{$_('hero.title')}</h1>
			<p class="py-8 text-lg text-white/70 sm:text-xl">{$_('hero.subtitle')}</p>
			<div class="flex flex-col justify-center gap-4 sm:flex-row">
				<a
					href="https://github.com/arktosmos/mhaoltube/releases"
					class="btn btn-lg btn-primary"
					target="_blank"
					rel="noopener noreferrer"
				>
					<svg
						xmlns="http://www.w3.org/2000/svg"
						fill="none"
						viewBox="0 0 24 24"
						stroke-width="1.5"
						stroke="currentColor"
						class="h-5 w-5"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							d="M3 16.5v2.25A2.25 2.25 0 0 0 5.25 21h13.5A2.25 2.25 0 0 0 21 18.75V16.5M16.5 12 12 16.5m0 0L7.5 12m4.5 4.5V3"
						/>
					</svg>
					{$_('hero.cta')}
				</a>
				<a
					href="https://github.com/arktosmos/mhaoltube"
					class="btn btn-outline btn-lg"
					target="_blank"
					rel="noopener noreferrer"
				>
					<svg
						xmlns="http://www.w3.org/2000/svg"
						viewBox="0 0 24 24"
						fill="currentColor"
						class="h-5 w-5"
					>
						<path
							d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0 0 24 12c0-6.63-5.37-12-12-12z"
						/>
					</svg>
					{$_('hero.source')}
				</a>
			</div>
		</div>
	</div>
</section>
