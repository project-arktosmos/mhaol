import type { Meta, StoryObj } from '@storybook/svelte';
import LibraryModalContent from 'ui-lib/components/libraries/LibraryModalContent.svelte';

const meta = {
	title: 'Libraries/LibraryModalContent',
	component: LibraryModalContent,
	tags: ['autodocs']
} satisfies Meta<typeof LibraryModalContent>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
