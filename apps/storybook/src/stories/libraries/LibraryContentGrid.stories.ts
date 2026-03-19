import type { Meta, StoryObj } from '@storybook/svelte';
import LibraryContentGrid from 'ui-lib/components/libraries/LibraryContentGrid.svelte';

const meta = {
	title: 'Libraries/LibraryContentGrid',
	component: LibraryContentGrid,
	tags: ['autodocs'],
	args: {
		title: 'My Library',
		items: [
			{ id: 1, title: 'Video One', thumbnailUrl: 'https://picsum.photos/320/180?1', duration: '5:30' },
			{ id: 2, title: 'Video Two', thumbnailUrl: 'https://picsum.photos/320/180?2', duration: '10:15' },
			{ id: 3, title: 'Video Three', thumbnailUrl: 'https://picsum.photos/320/180?3', duration: '3:45' }
		],
		activeDownloadMap: {}
	}
} satisfies Meta<typeof LibraryContentGrid>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
