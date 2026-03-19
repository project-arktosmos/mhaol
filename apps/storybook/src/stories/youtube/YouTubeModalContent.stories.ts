import type { Meta, StoryObj } from '@storybook/svelte';
import YouTubeModalContent from 'ui-lib/components/youtube/YouTubeModalContent.svelte';

const meta = {
	title: 'YouTube/YouTubeModalContent',
	component: YouTubeModalContent,
	tags: ['autodocs']
} satisfies Meta<typeof YouTubeModalContent>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
