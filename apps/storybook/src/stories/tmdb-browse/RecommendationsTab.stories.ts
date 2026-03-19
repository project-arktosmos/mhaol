import type { Meta, StoryObj } from '@storybook/svelte';
import RecommendationsTab from 'ui-lib/components/tmdb-browse/RecommendationsTab.svelte';

const meta = {
	title: 'TmdbBrowse/RecommendationsTab',
	component: RecommendationsTab,
	tags: ['autodocs'],
	args: {
		linkedItems: [],
		recommendations: [],
		page: 1,
		totalPages: 1,
		loading: false,
		onload: () => {}
	}
} satisfies Meta<typeof RecommendationsTab>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
