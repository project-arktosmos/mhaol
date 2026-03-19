import type { Meta, StoryObj } from '@storybook/svelte';
import TorrentListItem from 'ui-lib/components/torrent/TorrentListItem.svelte';

const meta = {
	title: 'Torrent/TorrentListItem',
	component: TorrentListItem,
	tags: ['autodocs']
} satisfies Meta<typeof TorrentListItem>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Downloading: Story = {
	args: {
		torrent: {
			infoHash: 'abc123def456',
			name: 'Big.Buck.Bunny.1080p.mkv',
			size: 1073741824,
			progress: 0.45,
			downloadSpeed: 2621440,
			uploadSpeed: 524288,
			peers: 12,
			seeds: 48,
			state: 'downloading',
			addedAt: 1710000000,
			eta: 1800,
			outputPath: '/downloads'
		}
	}
};

export const Completed: Story = {
	args: {
		torrent: {
			infoHash: 'def789abc012',
			name: 'Sintel.2010.4K.mp4',
			size: 4294967296,
			progress: 1.0,
			downloadSpeed: 0,
			uploadSpeed: 131072,
			peers: 3,
			seeds: 120,
			state: 'seeding',
			addedAt: 1709900000,
			eta: null,
			outputPath: '/downloads'
		}
	}
};
