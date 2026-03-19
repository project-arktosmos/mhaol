import type { Meta, StoryObj } from '@storybook/svelte';
import YouTubeUrlInput from 'ui-lib/components/youtube/YouTubeUrlInput.svelte';

const meta = {
	title: 'YouTube/YouTubeUrlInput',
	component: YouTubeUrlInput,
	tags: ['autodocs']
} satisfies Meta<typeof YouTubeUrlInput>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
