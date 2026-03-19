import type { Meta, StoryObj } from '@storybook/svelte';
import SignalingServerStatus from 'ui-lib/components/signaling/SignalingServerStatus.svelte';

const meta = {
	title: 'Signaling/SignalingServerStatus',
	component: SignalingServerStatus,
	tags: ['autodocs']
} satisfies Meta<typeof SignalingServerStatus>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
