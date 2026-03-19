import type { Meta, StoryObj } from '@storybook/svelte';
import PlayerVideo from 'ui-lib/components/player/PlayerVideo.svelte';

const meta = {
	title: 'Player/PlayerVideo',
	component: PlayerVideo,
	tags: ['autodocs']
} satisfies Meta<typeof PlayerVideo>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Idle: Story = {
	args: {
		file: {
			id: 'file-1',
			type: 'library',
			name: 'Big Buck Bunny.mp4',
			outputPath: '/media/videos/Big Buck Bunny.mp4',
			mode: 'video',
			format: 'mp4',
			videoFormat: 'h264',
			thumbnailUrl: null,
			durationSeconds: 596,
			size: 158008374,
			completedAt: '2024-01-15T10:30:00Z'
		},
		connectionState: 'idle',
		positionSecs: 0,
		durationSecs: null
	}
};

export const Connecting: Story = {
	args: {
		file: {
			id: 'file-2',
			type: 'library',
			name: 'Sample Video.mkv',
			outputPath: '/media/videos/Sample Video.mkv',
			mode: 'video',
			format: 'mkv',
			videoFormat: 'h264',
			thumbnailUrl: null,
			durationSeconds: 300,
			size: 52428800,
			completedAt: '2024-01-15T10:30:00Z'
		},
		connectionState: 'connecting',
		positionSecs: 0,
		durationSecs: null
	}
};

export const Error: Story = {
	args: {
		file: {
			id: 'file-3',
			type: 'library',
			name: 'Failed Stream.mp4',
			outputPath: '/media/videos/Failed Stream.mp4',
			mode: 'video',
			format: 'mp4',
			videoFormat: 'h264',
			thumbnailUrl: null,
			durationSeconds: 120,
			size: 10485760,
			completedAt: '2024-01-15T10:30:00Z'
		},
		connectionState: 'error',
		positionSecs: 0,
		durationSecs: null
	}
};

export const AudioMode: Story = {
	args: {
		file: {
			id: 'file-4',
			type: 'library',
			name: 'Song.mp3',
			outputPath: '/media/audio/Song.mp3',
			mode: 'audio',
			format: 'mp3',
			videoFormat: null,
			thumbnailUrl: null,
			durationSeconds: 240,
			size: 5242880,
			completedAt: '2024-01-15T10:30:00Z'
		},
		connectionState: 'idle',
		positionSecs: 0,
		durationSecs: null
	}
};
