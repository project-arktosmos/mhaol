import type { Meta, StoryObj } from '@storybook/svelte';
import LyricsPanel from 'ui-lib/components/player/LyricsPanel.svelte';

const meta = {
	title: 'Player/LyricsPanel',
	component: LyricsPanel,
	tags: ['autodocs']
} satisfies Meta<typeof LyricsPanel>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
	args: {
		currentFile: {
			id: 'file-1',
			type: 'library',
			name: 'Bohemian Rhapsody.flac',
			outputPath: '/media/audio/Bohemian Rhapsody.flac',
			mode: 'audio',
			format: 'flac',
			videoFormat: null,
			thumbnailUrl: null,
			durationSeconds: 355,
			size: 31457280,
			completedAt: '2024-01-15T10:30:00Z'
		},
		positionSecs: 45
	}
};

export const NoFile: Story = {
	args: {
		currentFile: null,
		positionSecs: 0
	}
};
