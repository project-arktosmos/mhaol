<script lang="ts">
	import classNames from 'classnames';
	import { ThemeColors, ThemeSizes } from '$types/core.type';
	import type { Snippet } from 'svelte';

	let {
		label = '',
		href = '',
		color = ThemeColors.Primary,
		outline = false,
		disabled = false,
		wide = false,
		tall = false,
		size = ThemeSizes.Medium,
		target = '_self',
		join = false,
		classes = '',
		onclick,
		ontouchstart,
		ontouchend,
		onmousedown,
		onmouseup,
		onmouseenter,
		onmouseleave,
		children
	}: {
		label?: string;
		href?: string;
		color?: ThemeColors;
		outline?: boolean;
		disabled?: boolean;
		wide?: boolean;
		tall?: boolean;
		size?: ThemeSizes;
		target?: string;
		join?: boolean;
		classes?: string;
		onclick?: () => void;
		ontouchstart?: (e: TouchEvent) => void;
		ontouchend?: (e: TouchEvent) => void;
		onmousedown?: (e: MouseEvent) => void;
		onmouseup?: (e: MouseEvent) => void;
		onmouseenter?: () => void;
		onmouseleave?: () => void;
		children?: Snippet;
	} = $props();

	const typesToClasses: Record<string, string> = {
		primary: 'btn-primary',
		secondary: 'btn-secondary',
		tertiary: 'btn-tertiary',
		success: 'btn-success',
		error: 'btn-error',
		info: 'btn-info',
		warning: 'btn-warning',
		neutral: 'btn-neutral'
	};

	const sizesToClasses: Record<string, string> = {
		sm: 'btn-sm',
		md: 'btn-md',
		lg: 'btn-lg'
	};

	let wrapperClasses = $derived(
		classNames(
			'btn',
			'relative',
			typesToClasses[color || ThemeColors.Neutral],
			sizesToClasses[size || 'md'],
			outline ? 'btn-outline' : null,
			wide ? 'w-full' : null,
			tall ? 'h-full' : null,
			join ? 'join-item' : null,
			classes,
			'pointer',
			'justify-center flex'
		)
	);

	function handleTouchStart(e: TouchEvent) {
		e.preventDefault();
		ontouchstart?.(e);
	}

	function handleTouchEnd(e: TouchEvent) {
		e.preventDefault();
		ontouchend?.(e);
	}
</script>

{#if href && !disabled}
	<a {href} class={wrapperClasses} {target}>
		{#if label}
			{label}
		{:else if children}
			{@render children()}
		{/if}
	</a>
{:else}
	<button
		ontouchstart={handleTouchStart}
		ontouchend={handleTouchEnd}
		onmousedown={(e) => onmousedown?.(e)}
		onmouseup={(e) => onmouseup?.(e)}
		onmouseenter={() => onmouseenter?.()}
		onmouseleave={() => onmouseleave?.()}
		onclick={() => onclick?.()}
		{disabled}
		class={wrapperClasses}
	>
		{#if label}
			{label}
		{:else if children}
			{@render children()}
		{/if}
	</button>
{/if}
