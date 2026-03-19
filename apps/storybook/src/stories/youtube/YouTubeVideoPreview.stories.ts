import type { Meta, StoryObj } from '@storybook/svelte';
import YouTubeVideoPreview from 'ui-lib/components/youtube/YouTubeVideoPreview.svelte';

const meta = {
	title: 'YouTube/YouTubeVideoPreview',
	component: YouTubeVideoPreview,
	tags: ['autodocs']
} satisfies Meta<typeof YouTubeVideoPreview>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
