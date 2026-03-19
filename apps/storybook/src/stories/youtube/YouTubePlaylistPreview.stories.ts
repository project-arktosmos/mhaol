import type { Meta, StoryObj } from '@storybook/svelte';
import YouTubePlaylistPreview from 'ui-lib/components/youtube/YouTubePlaylistPreview.svelte';

const meta = {
	title: 'YouTube/YouTubePlaylistPreview',
	component: YouTubePlaylistPreview,
	tags: ['autodocs']
} satisfies Meta<typeof YouTubePlaylistPreview>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
