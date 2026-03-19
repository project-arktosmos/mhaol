import type { Meta, StoryObj } from '@storybook/svelte';
import LibraryAddForm from 'ui-lib/components/libraries/LibraryAddForm.svelte';

const meta = {
	title: 'Libraries/LibraryAddForm',
	component: LibraryAddForm,
	tags: ['autodocs']
} satisfies Meta<typeof LibraryAddForm>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
