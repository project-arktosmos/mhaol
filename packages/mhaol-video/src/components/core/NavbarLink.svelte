<script lang="ts">
	import classNames from 'classnames';
	import { page } from '$app/stores';

	interface Props {
		href: string;
		label: string;
		classes?: string;
	}

	let { href, label, classes = '' }: Props = $props();

	let isActive = $derived(
		href === '/'
			? $page.url.pathname === '/'
			: $page.url.pathname === href || $page.url.pathname.startsWith(href + '/')
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
