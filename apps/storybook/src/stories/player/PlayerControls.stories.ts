import type { Meta, StoryObj } from '@storybook/svelte';
import PlayerControls from 'ui-lib/components/player/PlayerControls.svelte';

const meta = {
	title: 'Player/PlayerControls',
	component: PlayerControls,
	tags: ['autodocs']
} satisfies Meta<typeof PlayerControls>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Streaming: Story = {
	args: {
		positionSecs: 45,
		durationSecs: 300,
		connectionState: 'streaming',
		isVideo: false
	}
};

export const Idle: Story = {
	args: {
		positionSecs: 0,
		durationSecs: null,
		connectionState: 'idle',
		isVideo: false
	}
};

export const Connecting: Story = {
	args: {
		positionSecs: 0,
		durationSecs: null,
		connectionState: 'connecting',
		isVideo: false
	}
};

export const VideoMode: Story = {
	args: {
		positionSecs: 120,
		durationSecs: 600,
		connectionState: 'streaming',
		isVideo: true
	}
};
