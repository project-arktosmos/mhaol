import type { Meta, StoryObj } from '@storybook/svelte';
import CloudLibraryModalContent from 'ui-lib/components/cloud/CloudLibraryModalContent.svelte';

const meta = {
	title: 'Cloud/CloudLibraryModalContent',
	component: CloudLibraryModalContent,
	tags: ['autodocs']
} satisfies Meta<typeof CloudLibraryModalContent>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
