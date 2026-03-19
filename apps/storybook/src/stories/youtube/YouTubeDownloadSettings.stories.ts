import type { Meta, StoryObj } from '@storybook/svelte';
import YouTubeDownloadSettings from 'ui-lib/components/youtube/YouTubeDownloadSettings.svelte';

const meta = {
	title: 'YouTube/YouTubeDownloadSettings',
	component: YouTubeDownloadSettings,
	tags: ['autodocs']
} satisfies Meta<typeof YouTubeDownloadSettings>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
