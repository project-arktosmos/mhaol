import type { Meta, StoryObj } from '@storybook/svelte';
import PopularTab from 'ui-lib/components/tmdb-browse/PopularTab.svelte';

const meta = {
	title: 'TmdbBrowse/PopularTab',
	component: PopularTab,
	tags: ['autodocs'],
	args: {
		movies: [],
		tvShows: [],
		pages: { movies: { page: 1, totalPages: 1 }, tvShows: { page: 1, totalPages: 1 } },
		loading: false,
		onloadmoviepage: () => {},
		onloadtvshowpage: () => {},
		onselect: () => {}
	}
} satisfies Meta<typeof PopularTab>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
