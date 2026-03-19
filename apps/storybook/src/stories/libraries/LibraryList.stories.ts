import type { Meta, StoryObj } from '@storybook/svelte';
import LibraryList from 'ui-lib/components/libraries/LibraryList.svelte';

const meta = {
	title: 'Libraries/LibraryList',
	component: LibraryList,
	tags: ['autodocs']
} satisfies Meta<typeof LibraryList>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
