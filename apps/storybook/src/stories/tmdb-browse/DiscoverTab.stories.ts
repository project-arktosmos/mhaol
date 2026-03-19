import type { Meta, StoryObj } from '@storybook/svelte';
import DiscoverTab from 'ui-lib/components/tmdb-browse/DiscoverTab.svelte';

const meta = {
	title: 'TmdbBrowse/DiscoverTab',
	component: DiscoverTab,
	tags: ['autodocs'],
	args: {
		movies: [],
		tvShows: [],
		pages: { movies: { page: 1, totalPages: 1 }, tvShows: { page: 1, totalPages: 1 } },
		genres: { movie: [], tv: [] },
		loading: false,
		ondiscovermoviepage: () => {},
		ondiscovertvshowpage: () => {},
		ondiscovermovies: () => {},
		ondiscovertvshows: () => {},
		onselect: () => {}
	}
} satisfies Meta<typeof DiscoverTab>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
