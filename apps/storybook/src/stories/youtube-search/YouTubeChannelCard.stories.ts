import type { Meta, StoryObj } from '@storybook/svelte';
import YouTubeChannelCard from 'ui-lib/components/youtube-search/YouTubeChannelCard.svelte';

const meta = {
	title: 'YouTubeSearch/YouTubeChannelCard',
	component: YouTubeChannelCard,
	tags: ['autodocs'],
	argTypes: {
		subscribed: { control: 'boolean' }
	}
} satisfies Meta<typeof YouTubeChannelCard>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
	args: {
		channel: {
			type: 'channel',
			channelId: 'UCuAXFkgsw1L7xaCfnd5JJOw',
			name: 'Rick Astley',
			thumbnail: 'https://yt3.googleusercontent.com/example/photo.jpg',
			url: '/channel/UCuAXFkgsw1L7xaCfnd5JJOw',
			subscriberText: '3.5M subscribers',
			videoCountText: '120 videos',
			description: 'Official YouTube channel for Rick Astley.',
			verified: true
		},
		subscribed: false,
		onclick: (channel: unknown) => console.log('clicked:', channel),
		onsubscribe: (channel: unknown) => console.log('subscribe:', channel)
	}
};

export const Subscribed: Story = {
	args: {
		channel: {
			type: 'channel',
			channelId: 'UCSJ4gkVC6NrvII8umztf0Ow',
			name: 'Lofi Girl',
			thumbnail: 'https://yt3.googleusercontent.com/example/lofi.jpg',
			url: '/channel/UCSJ4gkVC6NrvII8umztf0Ow',
			subscriberText: '14M subscribers',
			videoCountText: '850 videos',
			description: 'Welcome to the Lofi Girl channel.',
			verified: true
		},
		subscribed: true,
		onclick: (channel: unknown) => console.log('clicked:', channel),
		onsubscribe: (channel: unknown) => console.log('unsubscribe:', channel)
	}
};
