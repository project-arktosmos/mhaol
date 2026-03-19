import type { Meta, StoryObj } from '@storybook/svelte';
import YouTubeSearchResultCard from 'ui-lib/components/youtube-search/YouTubeSearchResultCard.svelte';

const meta = {
	title: 'YouTubeSearch/YouTubeSearchResultCard',
	component: YouTubeSearchResultCard,
	tags: ['autodocs']
} satisfies Meta<typeof YouTubeSearchResultCard>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
	args: {
		item: {
			videoId: 'dQw4w9WgXcQ',
			type: 'stream',
			url: 'https://www.youtube.com/watch?v=dQw4w9WgXcQ',
			title: 'Rick Astley - Never Gonna Give You Up (Official Music Video)',
			thumbnail: 'https://i.ytimg.com/vi/dQw4w9WgXcQ/hqdefault.jpg',
			duration: 213,
			durationText: '3:33',
			views: 1500000000,
			viewsText: '1.5B views',
			uploadedDate: '15 years ago',
			uploaderName: 'Rick Astley',
			uploaderUrl: '/channel/UCuAXFkgsw1L7xaCfnd5JJOw',
			uploaderAvatar: '',
			uploaderVerified: true
		},
		onselect: (item: unknown) => console.log('selected:', item)
	}
};

export const LiveStream: Story = {
	args: {
		item: {
			videoId: 'live123',
			type: 'stream',
			url: 'https://www.youtube.com/watch?v=live123',
			title: 'Lofi Hip Hop Radio - Beats to Relax/Study To',
			thumbnail: 'https://i.ytimg.com/vi/jfKfPfyJRdk/hqdefault.jpg',
			duration: -1,
			durationText: '',
			views: 42000,
			viewsText: '42K watching',
			uploadedDate: '',
			uploaderName: 'Lofi Girl',
			uploaderUrl: '/channel/UCSJ4gkVC6NrvII8umztf0Ow',
			uploaderAvatar: '',
			uploaderVerified: true
		},
		onselect: (item: unknown) => console.log('selected:', item)
	}
};
