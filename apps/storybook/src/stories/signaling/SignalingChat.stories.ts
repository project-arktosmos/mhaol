import type { Meta, StoryObj } from '@storybook/svelte';
import SignalingChat from 'ui-lib/components/signaling/SignalingChat.svelte';

const meta = {
	title: 'Signaling/SignalingChat',
	component: SignalingChat,
	tags: ['autodocs']
} satisfies Meta<typeof SignalingChat>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
