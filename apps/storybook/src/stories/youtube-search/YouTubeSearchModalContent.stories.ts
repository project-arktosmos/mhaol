import type { Meta, StoryObj } from '@storybook/svelte';
import YouTubeSearchModalContent from 'ui-lib/components/youtube-search/YouTubeSearchModalContent.svelte';

const meta = {
	title: 'YouTubeSearch/YouTubeSearchModalContent',
	component: YouTubeSearchModalContent,
	tags: ['autodocs']
} satisfies Meta<typeof YouTubeSearchModalContent>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
