import type { Meta, StoryObj } from '@storybook/svelte';
import BrowseHeader from 'ui-lib/components/browse/BrowseHeader.svelte';

const meta = {
	title: 'Browse/BrowseHeader',
	component: BrowseHeader,
	tags: ['autodocs'],
	argTypes: {
		title: { control: 'text' },
		count: { control: 'number' },
		countLabel: { control: 'text' }
	}
} satisfies Meta<typeof BrowseHeader>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
	args: {
		title: 'Popular Albums',
		count: 42,
		countLabel: 'albums'
	}
};

export const NoCount: Story = {
	args: {
		title: 'Photo Gallery'
	}
};

export const ZeroCount: Story = {
	args: {
		title: 'Videogames',
		count: 0,
		countLabel: 'games'
	}
};

export const LargeCount: Story = {
	args: {
		title: 'All Items',
		count: 12483,
		countLabel: 'items'
	}
};
