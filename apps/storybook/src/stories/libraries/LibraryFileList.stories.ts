import type { Meta, StoryObj } from '@storybook/svelte';
import LibraryFileList from 'ui-lib/components/libraries/LibraryFileList.svelte';

const meta = {
	title: 'Libraries/LibraryFileList',
	component: LibraryFileList,
	tags: ['autodocs'],
	args: {
		files: [
			{ id: 1, libraryId: 1, path: '/media/file.mp4', name: 'file.mp4', size: 1048576, mediaType: 'video' },
			{ id: 2, libraryId: 1, path: '/media/song.mp3', name: 'song.mp3', size: 524288, mediaType: 'audio' }
		],
		loading: false,
		error: ''
	}
} satisfies Meta<typeof LibraryFileList>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
export const Loading: Story = { args: { files: [], loading: true, error: '' } };
export const Error: Story = { args: { files: [], loading: false, error: 'Failed to load files' } };
