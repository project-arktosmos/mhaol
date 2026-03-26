<script lang="ts">
	import QueueMonitorContent from 'ui-lib/components/queue/QueueMonitorContent.svelte';
</script>

<div class="flex-1 overflow-y-auto p-6">
	<div class="mb-6">
		<h1 class="text-3xl font-bold">Queue</h1>
		<p class="text-sm opacity-70">Real-time task queue status</p>
	</div>
	<QueueMonitorContent />
</div>
