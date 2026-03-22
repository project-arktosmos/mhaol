import type { Meta, StoryObj } from '@storybook/svelte';
import BrowseGrid from 'ui-lib/components/browse/BrowseGrid.svelte';

const meta = {
	title: 'Browse/BrowseGrid',
	component: BrowseGrid,
	tags: ['autodocs'],
	argTypes: {
		loading: { control: 'boolean' },
		error: { control: 'text' },
		emptyTitle: { control: 'text' },
		emptySubtitle: { control: 'text' },
		page: { control: 'number' },
		totalPages: { control: 'number' }
	}
} satisfies Meta<typeof BrowseGrid>;

export default meta;
type Story = StoryObj<typeof meta>;

const mockItems = Array.from({ length: 18 }, (_, i) => ({
	id: `item-${i}`,
	title: `Item ${i + 1}`,
	imageUrl: null
}));

export const Default: Story = {
	args: {
		items: mockItems
	}
};

export const Loading: Story = {
	args: {
		items: [],
		loading: true
	}
};

export const Error: Story = {
	args: {
		items: [],
		error: 'Failed to fetch items from server'
	}
};

export const Empty: Story = {
	args: {
		items: [],
		emptyTitle: 'No items found',
		emptySubtitle: 'Try adjusting your search filters'
	}
};

export const WithPagination: Story = {
	args: {
		items: mockItems.slice(0, 6),
		page: 2,
		totalPages: 10
	}
};

export const FewItems: Story = {
	args: {
		items: mockItems.slice(0, 3)
	}
};
