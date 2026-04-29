<script lang="ts">
	import classNames from 'classnames';

	interface Props {
		label: string;
		value: string | number | null | undefined;
		hint?: string | null;
		tone?: 'neutral' | 'success' | 'warning' | 'error';
		mono?: boolean;
	}

	let { label, value, hint = null, tone = 'neutral', mono = false }: Props = $props();

	const toneClass = (t: Props['tone']) =>
		classNames({
			'border-base-content/10': t === 'neutral',
			'border-success/40': t === 'success',
			'border-warning/40': t === 'warning',
			'border-error/40': t === 'error'
		});

	const valueClass = (t: Props['tone'], m: boolean) =>
		classNames('text-2xl font-semibold break-words', {
			'text-base-content': t === 'neutral',
			'text-success': t === 'success',
			'text-warning': t === 'warning',
			'text-error': t === 'error',
			'font-mono text-base': m
		});
</script>

<div class={classNames('flex flex-col gap-2 rounded-box border bg-base-200 p-4', toneClass(tone))}>
	<div class="text-xs tracking-wide text-base-content/60 uppercase">{label}</div>
	<div class={valueClass(tone, mono)}>
		{value ?? '—'}
	</div>
	{#if hint}
		<div class="text-xs text-base-content/50">{hint}</div>
	{/if}
</div>
