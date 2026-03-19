import type { Meta, StoryObj } from '@storybook/svelte';
import PeerLibraryList from 'ui-lib/components/peer-libraries/PeerLibraryList.svelte';

const meta = {
	title: 'PeerLibraries/PeerLibraryList',
	component: PeerLibraryList,
	tags: ['autodocs'],
	args: {
		peerId: 'peer-abc-123',
		peerData: {
			libraries: [
				{ id: 1, name: 'Movies', fileCount: 25 },
				{ id: 2, name: 'Music', fileCount: 100 }
			]
		}
	}
} satisfies Meta<typeof PeerLibraryList>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
