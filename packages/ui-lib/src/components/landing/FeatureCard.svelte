<script lang="ts">
	let {
		title,
		description,
		icon
	}: {
		title: string;
		description: string;
		icon: string;
	} = $props();
</script>

<div class="card bg-base-100 shadow-md transition-shadow hover:shadow-lg">
	<div class="card-body">
		<div class="mb-2 flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10">
			<svg
				xmlns="http://www.w3.org/2000/svg"
				fill="none"
				viewBox="0 0 24 24"
				stroke-width="1.5"
				stroke="currentColor"
				class="h-5 w-5 text-primary"
			>
				<path stroke-linecap="round" stroke-linejoin="round" d={icon} />
			</svg>
		</div>
		<h3 class="card-title text-lg">{title}</h3>
		<p class="text-sm text-base-content/60">{description}</p>
	</div>
</div>
