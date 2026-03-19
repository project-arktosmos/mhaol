import type { Meta, StoryObj } from '@storybook/svelte';
import MovieCard from 'ui-lib/components/media/MovieCard.svelte';

const meta = {
	title: 'Media/MovieCard',
	component: MovieCard,
	tags: ['autodocs']
} satisfies Meta<typeof MovieCard>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
	args: {
		item: {
			id: '1',
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
		}
	}
};

export const WithMetadata: Story = {
	args: {
		item: {
			id: '1',
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
		metadata: {
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
			cast: [{ name: 'Leonardo DiCaprio' }, { name: 'Joseph Gordon-Levitt' }]
		}
	}
};

export const Selected: Story = {
	args: {
		item: {
			id: '1',
			libraryId: 'lib-1',
			name: 'The Matrix.mkv',
			extension: '.mkv',
			path: '/media/movies/The Matrix.mkv',
			categoryId: 'movies',
			mediaTypeId: 'video',
			createdAt: '2024-01-15T10:30:00Z',
			links: {
				tmdb: { serviceId: '603', seasonNumber: null, episodeNumber: null }
			}
		},
		metadata: {
			title: 'The Matrix',
			posterUrl: 'https://picsum.photos/seed/matrix/300/450',
			releaseYear: '1999',
			voteAverage: 8.7,
			genres: ['Action', 'Science Fiction'],
			director: 'The Wachowskis',
			overview: 'A computer hacker learns about the true nature of reality.',
			runtime: '2h 16m',
			tagline: 'Welcome to the Real World.',
			cast: [{ name: 'Keanu Reeves' }, { name: 'Laurence Fishburne' }]
		},
		selected: true
	}
};

export const Loading: Story = {
	args: {
		item: {
			id: '1',
			libraryId: 'lib-1',
			name: 'Movie.mkv',
			extension: '.mkv',
			path: '/media/movies/Movie.mkv',
			categoryId: 'movies',
			mediaTypeId: 'video',
			createdAt: '2024-01-15T10:30:00Z',
			links: {
				tmdb: { serviceId: '12345', seasonNumber: null, episodeNumber: null }
			}
		},
		loading: true
	}
};
