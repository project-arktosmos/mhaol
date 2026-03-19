import type { Meta, StoryObj } from '@storybook/svelte';
import YouTubePreviewModal from 'ui-lib/components/libraries/YouTubePreviewModal.svelte';

const meta = {
	title: 'Libraries/YouTubePreviewModal',
	component: YouTubePreviewModal,
	tags: ['autodocs'],
	args: {
		file: { id: 1, libraryId: 1, path: '/media/file.mp4', name: 'file.mp4', size: 1048576, mediaType: 'video' },
		videoId: 'dQw4w9WgXcQ',
		onclose: () => {}
	}
} satisfies Meta<typeof YouTubePreviewModal>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
