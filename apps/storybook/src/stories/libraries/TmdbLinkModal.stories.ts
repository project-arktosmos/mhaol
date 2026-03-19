import type { Meta, StoryObj } from '@storybook/svelte';
import TmdbLinkModal from 'ui-lib/components/libraries/TmdbLinkModal.svelte';

const meta = {
	title: 'Libraries/TmdbLinkModal',
	component: TmdbLinkModal,
	tags: ['autodocs'],
	args: {
		file: { id: 1, libraryId: 1, path: '/media/file.mp4', name: 'file.mp4', size: 1048576, mediaType: 'video' },
		files: [],
		type: 'movie',
		onlink: () => {},
		onclose: () => {}
	}
} satisfies Meta<typeof TmdbLinkModal>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
