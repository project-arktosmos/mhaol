import type { Meta, StoryObj } from '@storybook/svelte';
import YouTubeCard from 'ui-lib/components/media/YouTubeCard.svelte';

const meta = {
	title: 'Media/YouTubeCard',
	component: YouTubeCard,
	tags: ['autodocs']
} satisfies Meta<typeof YouTubeCard>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
	args: {
		item: {
			id: '1',
			libraryId: 'lib-1',
			name: 'YouTube Video.mp4',
			extension: '.mp4',
			path: '/media/youtube/YouTube Video.mp4',
			categoryId: null,
			mediaTypeId: 'video',
			createdAt: '2024-01-15T10:30:00Z',
			links: {
				youtube: { serviceId: 'dQw4w9WgXcQ', seasonNumber: null, episodeNumber: null }
			}
		}
	}
};

export const WithMetadata: Story = {
	args: {
		item: {
			id: '2',
			libraryId: 'lib-1',
			name: 'Tech Review.mp4',
			extension: '.mp4',
			path: '/media/youtube/Tech Review.mp4',
			categoryId: null,
			mediaTypeId: 'video',
			createdAt: '2024-01-15T10:30:00Z',
			links: {
				youtube: { serviceId: 'abc123def', seasonNumber: null, episodeNumber: null }
			}
		},
		metadata: {
			title: 'The Best Laptops of 2024 - Full Review',
			author_name: 'TechChannel',
			author_url: 'https://youtube.com/@TechChannel',
			thumbnail_url: 'https://picsum.photos/seed/techreview/480/360'
		}
	}
};

export const Selected: Story = {
	args: {
		item: {
			id: '3',
			libraryId: 'lib-1',
			name: 'Music Video.mp4',
			extension: '.mp4',
			path: '/media/youtube/Music Video.mp4',
			categoryId: null,
			mediaTypeId: 'video',
			createdAt: '2024-01-15T10:30:00Z',
			links: {
				youtube: { serviceId: 'xyz789abc', seasonNumber: null, episodeNumber: null }
			}
		},
		metadata: {
			title: 'Amazing Music Video - Official',
			author_name: 'MusicArtist',
			author_url: 'https://youtube.com/@MusicArtist',
			thumbnail_url: 'https://picsum.photos/seed/musicvid/480/360'
		},
		selected: true
	}
};
