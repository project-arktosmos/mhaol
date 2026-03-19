import type { Meta, StoryObj } from '@storybook/svelte';
import PeerLibraryFileList from 'ui-lib/components/peer-libraries/PeerLibraryFileList.svelte';

const meta = {
	title: 'PeerLibraries/PeerLibraryFileList',
	component: PeerLibraryFileList,
	tags: ['autodocs'],
	args: {
		files: [
			{ name: 'movie.mp4', size: 1048576 },
			{ name: 'song.mp3', size: 524288 },
			{ name: 'photo.jpg', size: 262144 }
		],
		loading: false
	}
} satisfies Meta<typeof PeerLibraryFileList>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
export const Loading: Story = { args: { files: [], loading: true } };
