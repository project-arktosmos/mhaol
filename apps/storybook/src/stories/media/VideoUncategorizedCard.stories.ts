import type { Meta, StoryObj } from '@storybook/svelte';
import VideoUncategorizedCard from 'ui-lib/components/media/VideoUncategorizedCard.svelte';

const meta = {
	title: 'Media/VideoUncategorizedCard',
	component: VideoUncategorizedCard,
	tags: ['autodocs']
} satisfies Meta<typeof VideoUncategorizedCard>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
	args: {
		item: {
			id: '1',
			libraryId: 'lib-1',
			name: 'Vacation Footage 2024.mp4',
			extension: '.mp4',
			path: '/media/videos/Vacation Footage 2024.mp4',
			categoryId: null,
			mediaTypeId: 'video',
			createdAt: '2024-01-15T10:30:00Z',
			links: {}
		}
	}
};

export const Selected: Story = {
	args: {
		item: {
			id: '2',
			libraryId: 'lib-1',
			name: 'Screen Recording.mkv',
			extension: '.mkv',
			path: '/media/videos/Screen Recording.mkv',
			categoryId: null,
			mediaTypeId: 'video',
			createdAt: '2024-02-20T14:00:00Z',
			links: {}
		},
		selected: true
	}
};
