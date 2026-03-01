<script lang="ts">
	import { torrentModalService } from '$services/torrent-modal.service';
</script>

<div class="flex flex-col items-center justify-center gap-4 p-6">
	<h1 class="text-2xl font-bold">Torrent Manager</h1>
	<p class="text-sm text-base-content/60">Download and manage torrents via magnet links</p>
	<button class="btn btn-primary" onclick={() => torrentModalService.open()}>
		Open Torrent Manager
	</button>
</div>
