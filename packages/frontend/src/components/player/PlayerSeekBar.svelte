<script lang="ts">
	import classNames from 'classnames';
	import { createEventDispatcher, onDestroy } from 'svelte';
	import { playerAdapter } from '$adapters/classes/player.adapter';

	export let positionSecs: number = 0;
	export let durationSecs: number | null = null;
	export let disabled: boolean = false;

	const dispatch = createEventDispatcher<{
		seek: { positionSecs: number };
		seekstart: void;
		seekend: void;
	}>();

	let isDragging = false;
	let dragPosition = 0;
	let trackElement: HTMLDivElement;

	$: progress =
		durationSecs && durationSecs > 0
			? ((isDragging ? dragPosition : positionSecs) / durationSecs) * 100
			: 0;

	$: displayPosition = isDragging ? dragPosition : positionSecs;

	function getPositionFromEvent(event: MouseEvent): number {
		if (!trackElement || !durationSecs) return 0;
		const rect = trackElement.getBoundingClientRect();
		const fraction = Math.max(0, Math.min(1, (event.clientX - rect.left) / rect.width));
		return fraction * durationSecs;
	}

	function handleMouseDown(event: MouseEvent): void {
		if (disabled || !durationSecs) return;
		isDragging = true;
		dragPosition = getPositionFromEvent(event);
		dispatch('seekstart');
		window.addEventListener('mousemove', handleMouseMove);
		window.addEventListener('mouseup', handleMouseUp);
	}

	function handleMouseMove(event: MouseEvent): void {
		if (!isDragging) return;
		dragPosition = getPositionFromEvent(event);
	}

	function handleMouseUp(event: MouseEvent): void {
		if (!isDragging) return;
		isDragging = false;
		const finalPosition = getPositionFromEvent(event);
		dispatch('seek', { positionSecs: finalPosition });
		dispatch('seekend');
		window.removeEventListener('mousemove', handleMouseMove);
		window.removeEventListener('mouseup', handleMouseUp);
	}

	onDestroy(() => {
		window.removeEventListener('mousemove', handleMouseMove);
		window.removeEventListener('mouseup', handleMouseUp);
	});
</script>

<div class={classNames('flex flex-col gap-0.5', { 'pointer-events-none opacity-50': disabled })}>
	<div
		bind:this={trackElement}
		class="group relative h-1.5 cursor-pointer rounded-full bg-base-300"
		role="slider"
		aria-label="Seek"
		aria-valuemin={0}
		aria-valuemax={durationSecs ?? 0}
		aria-valuenow={displayPosition}
		tabindex="0"
		on:mousedown={handleMouseDown}
	>
		<div class="absolute inset-y-0 left-0 rounded-full bg-primary" style:width="{progress}%"></div>

		<div
			class={classNames(
				'absolute top-1/2 h-3 w-3 -translate-x-1/2 -translate-y-1/2 rounded-full bg-primary shadow-md',
				{
					'scale-100': isDragging,
					'scale-0 group-hover:scale-100': !isDragging
				}
			)}
			style:left="{progress}%"
		></div>
	</div>

	<div class="flex justify-between font-mono text-[10px] leading-tight opacity-60">
		<span>{playerAdapter.formatDuration(displayPosition)}</span>
		<span>{playerAdapter.formatDuration(durationSecs)}</span>
	</div>
</div>
