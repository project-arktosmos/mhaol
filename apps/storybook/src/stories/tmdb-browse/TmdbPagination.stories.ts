import type { Meta, StoryObj } from '@storybook/svelte';
import TmdbPagination from 'ui-lib/components/tmdb-browse/TmdbPagination.svelte';

const meta = {
	title: 'TmdbBrowse/TmdbPagination',
	component: TmdbPagination,
	tags: ['autodocs'],
	args: {
		page: 1,
		totalPages: 10,
		loading: false,
		onpage: () => {}
	},
	argTypes: {
		page: { control: { type: 'range', min: 1, max: 100, step: 1 } },
		totalPages: { control: { type: 'range', min: 1, max: 100, step: 1 } }
	}
} satisfies Meta<typeof TmdbPagination>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
export const MiddlePage: Story = { args: { page: 5, totalPages: 10 } };
export const Loading: Story = { args: { page: 1, totalPages: 10, loading: true } };
