import type { Meta, StoryObj } from '@storybook/svelte';
import TmdbBrowseCard from 'ui-lib/components/tmdb-browse/TmdbBrowseCard.svelte';

const meta = {
	title: 'TmdbBrowse/TmdbBrowseCard',
	component: TmdbBrowseCard,
	tags: ['autodocs'],
	args: {
		movie: {
			id: 550,
			title: 'Fight Club',
			overview: 'An insomniac office worker and a devil-may-care soap maker form an underground fight club.',
			poster_path: '/pB8BM7pdSp6B6Ih7QZ4DrQ3PmJK.jpg',
			release_date: '1999-10-15',
			vote_average: 8.4
		}
	}
} satisfies Meta<typeof TmdbBrowseCard>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Movie: Story = {};

export const TvShow: Story = {
	args: {
		movie: undefined,
		tvShow: {
			id: 1396,
			name: 'Breaking Bad',
			overview: 'A high school chemistry teacher turned methamphetamine manufacturer.',
			poster_path: '/ggFHVNu6YYI5L9pCfOacjizRGt.jpg',
			first_air_date: '2008-01-20',
			vote_average: 8.9
		}
	}
};
