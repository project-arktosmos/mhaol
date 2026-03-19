import type { Meta, StoryObj } from '@storybook/svelte';
import MediaCard from 'ui-lib/components/media/MediaCard.svelte';

const meta = {
	title: 'Media/MediaCard',
	component: MediaCard,
	tags: ['autodocs']
} satisfies Meta<typeof MediaCard>;

export default meta;
type Story = StoryObj<typeof meta>;

export const VideoUncategorized: Story = {
	args: {
		item: {
			id: '1',
			libraryId: 'lib-1',
			name: 'Vacation Footage.mp4',
			extension: '.mp4',
			path: '/media/videos/Vacation Footage.mp4',
			categoryId: null,
			mediaTypeId: 'video',
			createdAt: '2024-01-15T10:30:00Z',
			links: {}
		}
	}
};

export const AudioUncategorized: Story = {
	args: {
		item: {
			id: '2',
			libraryId: 'lib-1',
			name: 'Recording.mp3',
			extension: '.mp3',
			path: '/media/audio/Recording.mp3',
			categoryId: null,
			mediaTypeId: 'audio',
			createdAt: '2024-01-15T10:30:00Z',
			links: {}
		}
	}
};

export const ImageUncategorized: Story = {
	args: {
		item: {
			id: '3',
			libraryId: 'lib-1',
			name: 'Photo.jpg',
			extension: '.jpg',
			path: '/media/images/Photo.jpg',
			categoryId: null,
			mediaTypeId: 'image',
			createdAt: '2024-01-15T10:30:00Z',
			links: {}
		}
	}
};

export const MovieWithTmdbLink: Story = {
	args: {
		item: {
			id: '4',
			libraryId: 'lib-1',
			name: 'Inception.mkv',
			extension: '.mkv',
			path: '/media/movies/Inception.mkv',
			categoryId: 'movies',
			mediaTypeId: 'video',
			createdAt: '2024-01-15T10:30:00Z',
			links: {
				tmdb: { serviceId: '27205', seasonNumber: null, episodeNumber: null }
			}
		},
		tmdbMetadata: {
			title: 'Inception',
			posterUrl: 'https://picsum.photos/seed/inception/300/450',
			releaseYear: '2010',
			voteAverage: 8.4,
			genres: ['Action', 'Science Fiction', 'Adventure'],
			director: 'Christopher Nolan',
			overview:
				'Cobb, a skilled thief who commits corporate espionage by infiltrating the subconscious of his targets is offered a chance to regain his old life.',
			runtime: '2h 28m',
			tagline: 'Your mind is the scene of the crime.',
			cast: []
		}
	}
};
