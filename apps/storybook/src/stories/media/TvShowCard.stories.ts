import type { Meta, StoryObj } from '@storybook/svelte';
import TvShowCard from 'ui-lib/components/media/TvShowCard.svelte';

const meta = {
	title: 'Media/TvShowCard',
	component: TvShowCard,
	tags: ['autodocs']
} satisfies Meta<typeof TvShowCard>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
	args: {
		item: {
			id: '1',
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
		}
	}
};

export const WithMetadata: Story = {
	args: {
		item: {
			id: '1',
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
		metadata: {
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
				'A high school chemistry teacher turned methamphetamine manufacturer partners with a former student.',
			tagline: 'All Hail the King.',
			cast: [{ name: 'Bryan Cranston' }, { name: 'Aaron Paul' }],
			seasons: []
		}
	}
};

export const Selected: Story = {
	args: {
		item: {
			id: '2',
			libraryId: 'lib-1',
			name: 'The Office S03E15.mkv',
			extension: '.mkv',
			path: '/media/tv/The Office S03E15.mkv',
			categoryId: 'tv',
			mediaTypeId: 'video',
			createdAt: '2024-01-15T10:30:00Z',
			links: {
				tmdb: { serviceId: '2316', seasonNumber: 3, episodeNumber: 15 }
			}
		},
		metadata: {
			name: 'The Office',
			posterUrl: 'https://picsum.photos/seed/theoffice/300/450',
			firstAirYear: '2005',
			lastAirYear: '2013',
			voteAverage: 8.6,
			status: 'Ended',
			genres: ['Comedy'],
			numberOfSeasons: 9,
			numberOfEpisodes: 201,
			createdBy: ['Greg Daniels'],
			overview: 'A mockumentary on a group of typical office workers.',
			tagline: '',
			cast: [{ name: 'Steve Carell' }],
			seasons: []
		},
		selected: true
	}
};
