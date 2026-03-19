<script lang="ts">
	import classNames from 'classnames';

	interface Props {
		href: string;
		label: string;
		currentPath: string;
		classes?: string;
	}

	let { href, label, currentPath, classes = '' }: Props = $props();

	let isActive = $derived(
		href === '/'
			? currentPath === '/'
			: currentPath === href || currentPath.startsWith(href + '/')
	);

	let linkClasses = $derived(
		classNames(
			'btn btn-ghost btn-sm',
			{
				'btn-active': isActive
			},
			classes
		)
	);
</script>

<a {href} class={linkClasses}>
	{label}
</a>
