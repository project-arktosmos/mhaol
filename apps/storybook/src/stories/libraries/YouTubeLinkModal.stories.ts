import type { Meta, StoryObj } from '@storybook/svelte';
import YouTubeLinkModal from 'ui-lib/components/libraries/YouTubeLinkModal.svelte';

const meta = {
	title: 'Libraries/YouTubeLinkModal',
	component: YouTubeLinkModal,
	tags: ['autodocs'],
	args: {
		file: { id: 1, libraryId: 1, path: '/media/file.mp4', name: 'file.mp4', size: 1048576, mediaType: 'video' },
		onlink: () => {},
		onclose: () => {}
	}
} satisfies Meta<typeof YouTubeLinkModal>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
