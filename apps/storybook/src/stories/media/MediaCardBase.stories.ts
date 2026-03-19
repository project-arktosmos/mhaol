import type { Meta, StoryObj } from '@storybook/svelte';
import MediaCardBase from 'ui-lib/components/media/MediaCardBase.svelte';

const meta = {
	title: 'Media/MediaCardBase',
	component: MediaCardBase,
	tags: ['autodocs']
} satisfies Meta<typeof MediaCardBase>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
	args: {
		item: {
			id: '1',
			libraryId: 'lib-1',
			name: 'Big Buck Bunny.mp4',
			extension: '.mp4',
			path: '/media/videos/Big Buck Bunny.mp4',
			categoryId: null,
			mediaTypeId: 'video',
			createdAt: '2024-01-15T10:30:00Z',
			links: {}
		}
	}
};

export const WithImage: Story = {
	args: {
		item: {
			id: '2',
			libraryId: 'lib-1',
			name: 'Inception.mkv',
			extension: '.mkv',
			path: '/media/movies/Inception.mkv',
			categoryId: 'movies',
			mediaTypeId: 'video',
			createdAt: '2024-01-15T10:30:00Z',
			links: {}
		},
		imageUrl: 'https://picsum.photos/seed/inception/400/300',
		imageAlt: 'Inception poster'
	}
};

export const Selected: Story = {
	args: {
		item: {
			id: '3',
			libraryId: 'lib-1',
			name: 'Selected Item.mp4',
			extension: '.mp4',
			path: '/media/videos/Selected Item.mp4',
			categoryId: null,
			mediaTypeId: 'video',
			createdAt: '2024-01-15T10:30:00Z',
			links: {}
		},
		selected: true
	}
};

export const Loading: Story = {
	args: {
		item: {
			id: '4',
			libraryId: 'lib-1',
			name: 'Loading Item.mp4',
			extension: '.mp4',
			path: '/media/videos/Loading Item.mp4',
			categoryId: null,
			mediaTypeId: 'video',
			createdAt: '2024-01-15T10:30:00Z',
			links: {}
		},
		loading: true
	}
};
