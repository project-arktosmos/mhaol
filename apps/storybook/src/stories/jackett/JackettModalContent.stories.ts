import type { Meta, StoryObj } from '@storybook/svelte';
import JackettModalContent from 'ui-lib/components/jackett/JackettModalContent.svelte';

const meta = {
	title: 'Jackett/JackettModalContent',
	component: JackettModalContent,
	tags: ['autodocs']
} satisfies Meta<typeof JackettModalContent>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
