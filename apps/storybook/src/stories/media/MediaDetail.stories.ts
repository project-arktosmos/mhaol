import type { Meta, StoryObj } from '@storybook/svelte';
import MediaDetail from 'ui-lib/components/media/MediaDetail.svelte';

const meta = {
	title: 'Media/MediaDetail',
	component: MediaDetail,
	tags: ['autodocs']
} satisfies Meta<typeof MediaDetail>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Video: Story = {
	args: {
		selection: {
			cardType: 'video',
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
			},
			tmdbMetadata: null,
			youtubeMetadata: null,
			musicbrainzMetadata: null,
			imageTags: [],
			imageTagging: false
		}
	}
};

export const Movie: Story = {
	args: {
		selection: {
			cardType: 'movie',
			item: {
				id: '2',
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
				cast: [{ name: 'Leonardo DiCaprio' }, { name: 'Joseph Gordon-Levitt' }]
			},
			youtubeMetadata: null,
			musicbrainzMetadata: null,
			imageTags: [],
			imageTagging: false
		}
	}
};

export const TvShow: Story = {
	args: {
		selection: {
			cardType: 'tv',
			item: {
				id: '3',
				libraryId: 'lib-1',
				name: 'Breaking Bad S01E01.mkv',
				extension: '.mkv',
				path: '/media/tv/Breaking Bad S01E01.mkv',
				categoryId: 'tv',
				mediaTypeId: 'video',
				createdAt: '2024-01-15T10:30:00Z',
				links: {
					tmdb: { serviceId: '1396', seasonNumber: 1, episodeNumber: 1 }
				}
			},
			tmdbMetadata: {
				name: 'Breaking Bad',
				posterUrl: 'https://picsum.photos/seed/breakingbad/300/450',
				firstAirYear: '2008',
				lastAirYear: '2013',
				voteAverage: 8.9,
				status: 'Ended',
				genres: ['Drama', 'Crime'],
				numberOfSeasons: 5,
				numberOfEpisodes: 62,
				createdBy: ['Vince Gilligan'],
				overview:
					'A high school chemistry teacher turned methamphetamine manufacturer.',
				tagline: 'All Hail the King.',
				cast: [{ name: 'Bryan Cranston' }, { name: 'Aaron Paul' }],
				seasons: []
			},
			youtubeMetadata: null,
			musicbrainzMetadata: null,
			imageTags: [],
			imageTagging: false
		}
	}
};

export const ImageWithTags: Story = {
	args: {
		selection: {
			cardType: 'image',
			item: {
				id: '4',
				libraryId: 'lib-1',
				name: 'Landscape.jpg',
				extension: '.jpg',
				path: '/media/images/Landscape.jpg',
				categoryId: null,
				mediaTypeId: 'image',
				createdAt: '2024-01-15T10:30:00Z',
				links: {}
			},
			tmdbMetadata: null,
			youtubeMetadata: null,
			musicbrainzMetadata: null,
			imageTags: [
				{ tag: 'landscape', score: 0.95 },
				{ tag: 'mountain', score: 0.88 }
			],
			imageTagging: false
		}
	}
};
