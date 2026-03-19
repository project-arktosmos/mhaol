import type { Meta, StoryObj } from '@storybook/svelte';
import SignalingConnectionControls from 'ui-lib/components/signaling/SignalingConnectionControls.svelte';

const meta = {
	title: 'Signaling/SignalingConnectionControls',
	component: SignalingConnectionControls,
	tags: ['autodocs']
} satisfies Meta<typeof SignalingConnectionControls>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
