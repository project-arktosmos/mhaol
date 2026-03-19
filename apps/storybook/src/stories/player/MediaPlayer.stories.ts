import type { Meta, StoryObj } from '@storybook/svelte';
import MediaPlayer from 'ui-lib/components/player/MediaPlayer.svelte';

const meta = {
	title: 'Player/MediaPlayer',
	component: MediaPlayer,
	tags: ['autodocs']
} satisfies Meta<typeof MediaPlayer>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Video: Story = {
	args: {
		source: {
			type: 'video',
			src: 'https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4',
			mimeType: 'video/mp4'
		},
		autoplay: false
	}
};

export const Audio: Story = {
	args: {
		source: {
			type: 'audio',
			src: 'https://www.soundhelix.com/examples/mp3/SoundHelix-Song-1.mp3',
			mimeType: 'audio/mpeg'
		},
		autoplay: false
	}
};

export const YouTube: Story = {
	args: {
		source: {
			type: 'youtube',
			videoId: 'dQw4w9WgXcQ',
			title: 'Sample YouTube Video'
		},
		autoplay: false
	}
};
