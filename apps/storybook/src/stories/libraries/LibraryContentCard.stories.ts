import type { Meta, StoryObj } from '@storybook/svelte';
import LibraryContentCard from 'ui-lib/components/libraries/LibraryContentCard.svelte';

const meta = {
	title: 'Libraries/LibraryContentCard',
	component: LibraryContentCard,
	tags: ['autodocs'],
	args: {
		item: {
			id: 1,
			title: 'Sample Video',
			thumbnailUrl: 'https://picsum.photos/320/180',
			duration: '12:34'
		},
		onclick: () => {}
	}
} satisfies Meta<typeof LibraryContentCard>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
