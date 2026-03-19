import type { Meta, StoryObj } from '@storybook/svelte';
import PlayerSeekBar from 'ui-lib/components/player/PlayerSeekBar.svelte';

const meta = {
	title: 'Player/PlayerSeekBar',
	component: PlayerSeekBar,
	tags: ['autodocs'],
	argTypes: {
		positionSecs: {
			control: { type: 'range', min: 0, max: 300, step: 1 }
		},
		durationSecs: {
			control: { type: 'range', min: 0, max: 600, step: 1 }
		},
		disabled: {
			control: 'boolean'
		}
	}
} satisfies Meta<typeof PlayerSeekBar>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
	args: { positionSecs: 45, durationSecs: 300, disabled: false }
};

export const AtStart: Story = {
	args: { positionSecs: 0, durationSecs: 180, disabled: false }
};

export const Midway: Story = {
	args: { positionSecs: 150, durationSecs: 300, disabled: false }
};

export const NearEnd: Story = {
	args: { positionSecs: 280, durationSecs: 300, disabled: false }
};

export const Disabled: Story = {
	args: { positionSecs: 60, durationSecs: 300, disabled: true }
};

export const NoDuration: Story = {
	args: { positionSecs: 0, durationSecs: null, disabled: false }
};
