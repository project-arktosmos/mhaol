import type { Meta, StoryObj } from '@storybook/svelte';
import YouTubeDownloadQueue from 'ui-lib/components/youtube/YouTubeDownloadQueue.svelte';

const meta = {
	title: 'YouTube/YouTubeDownloadQueue',
	component: YouTubeDownloadQueue,
	tags: ['autodocs']
} satisfies Meta<typeof YouTubeDownloadQueue>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
