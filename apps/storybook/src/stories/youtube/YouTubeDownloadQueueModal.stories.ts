import type { Meta, StoryObj } from '@storybook/svelte';
import YouTubeDownloadQueueModal from 'ui-lib/components/youtube/YouTubeDownloadQueueModal.svelte';

const meta = {
	title: 'YouTube/YouTubeDownloadQueueModal',
	component: YouTubeDownloadQueueModal,
	tags: ['autodocs'],
	argTypes: {
		open: { control: 'boolean' }
	}
} satisfies Meta<typeof YouTubeDownloadQueueModal>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Open: Story = { args: { open: true, onClose: () => {} } };
export const Closed: Story = { args: { open: false, onClose: () => {} } };
