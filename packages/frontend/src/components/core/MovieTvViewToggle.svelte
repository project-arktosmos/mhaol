<script lang="ts">
	import { Icon } from 'cloud-ui';
	import { movieTvViewModeService } from '$services/movie-tv-view-mode.service';

	const store = movieTvViewModeService.store;

	let isLandscape = $derived($store.mode === 'landscapes');

	function toggle() {
		movieTvViewModeService.toggle();
	}
</script>

<label
	class="btn swap swap-rotate btn-ghost btn-sm"
	title={isLandscape
		? 'Movie/TV cards: landscapes (click for covers)'
		: 'Movie/TV cards: covers (click for landscapes)'}
>
	<input type="checkbox" checked={isLandscape} onchange={toggle} />
	<span class="swap-off flex h-5 w-5 items-center justify-center">
		<Icon name="delapouite/portrait" size={20} />
	</span>
	<span class="swap-on flex h-5 w-5 items-center justify-center">
		<Icon name="delapouite/tv" size={20} />
	</span>
</label>
