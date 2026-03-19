import type { Meta, StoryObj } from '@storybook/svelte';
import MediaTypeCategoryModal from 'ui-lib/components/libraries/MediaTypeCategoryModal.svelte';

const meta = {
	title: 'Libraries/MediaTypeCategoryModal',
	component: MediaTypeCategoryModal,
	tags: ['autodocs'],
	args: {
		file: { id: 1, libraryId: 1, path: '/media/file.mp4', name: 'file.mp4', size: 1048576, mediaType: 'video' },
		onsave: () => {},
		onclose: () => {}
	}
} satisfies Meta<typeof MediaTypeCategoryModal>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
