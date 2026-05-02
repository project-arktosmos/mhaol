<script lang="ts">
	import classNames from 'classnames';
	import { playerAdapter } from '$adapters/classes/player.adapter';

	let {
		positionSecs = 0,
		durationSecs = null,
		disabled = false,
		onseek,
		onseekstart,
		onseekend
	}: {
		positionSecs?: number;
		durationSecs?: number | null;
		disabled?: boolean;
		onseek?: (positionSecs: number) => void;
		onseekstart?: () => void;
		onseekend?: () => void;
	} = $props();

	let isDragging = $state(false);
	let dragPosition = $state(0);
	let trackElement: HTMLDivElement;

	let progress = $derived(
		durationSecs && durationSecs > 0
			? ((isDragging ? dragPosition : positionSecs) / durationSecs) * 100
			: 0
	);

	let displayPosition = $derived(isDragging ? dragPosition : positionSecs);

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
		onseekstart?.();
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
		onseek?.(finalPosition);
		onseekend?.();
		window.removeEventListener('mousemove', handleMouseMove);
		window.removeEventListener('mouseup', handleMouseUp);
	}

	$effect(() => {
		return () => {
			window.removeEventListener('mousemove', handleMouseMove);
			window.removeEventListener('mouseup', handleMouseUp);
		};
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
		onmousedown={handleMouseDown}
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
