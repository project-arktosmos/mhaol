<script lang="ts">
	import { _ } from 'svelte-i18n';
</script>

<footer class="border-t border-base-300 bg-base-200 py-10">
	<div class="mx-auto max-w-6xl px-6">
		<div class="flex flex-col items-center gap-4 text-center">
			<span class="text-xl font-bold">Mhaoltube</span>
			<p class="text-sm text-base-content/50">{$_('footer.tagline')}</p>
			<div class="flex gap-4">
				<a
					href="https://github.com/arktosmos/mhaoltube"
					class="btn btn-ghost btn-sm"
					target="_blank"
					rel="noopener noreferrer"
				>
					GitHub
				</a>
				<a
					href="https://github.com/arktosmos/mhaoltube/releases"
					class="btn btn-ghost btn-sm"
					target="_blank"
					rel="noopener noreferrer"
				>
					Releases
				</a>
			</div>
			<p class="text-xs text-base-content/30">{$_('footer.built_with')}</p>
		</div>
	</div>
</footer>
