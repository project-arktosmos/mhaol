import type { Meta, StoryObj } from '@storybook/svelte';
import SignalingModalContent from 'ui-lib/components/signaling/SignalingModalContent.svelte';

const meta = {
	title: 'Signaling/SignalingModalContent',
	component: SignalingModalContent,
	tags: ['autodocs']
} satisfies Meta<typeof SignalingModalContent>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
