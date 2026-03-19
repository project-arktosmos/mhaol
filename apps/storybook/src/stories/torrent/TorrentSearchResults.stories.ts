import type { Meta, StoryObj } from '@storybook/svelte';
import TorrentSearchResults from 'ui-lib/components/torrent/TorrentSearchResults.svelte';

const meta = {
	title: 'Torrent/TorrentSearchResults',
	component: TorrentSearchResults,
	tags: ['autodocs']
} satisfies Meta<typeof TorrentSearchResults>;

export default meta;
type Story = StoryObj<typeof meta>;

export const WithResults: Story = {
	args: {
		results: [
			{
				id: '1',
				name: 'Ubuntu 24.04 Desktop AMD64',
				size: '4.7 GB',
				seeders: 1200,
				leechers: 45,
				magnetLink: 'magnet:?xt=urn:btih:abc123',
				infoHash: 'abc123',
				uploadDate: '2024-04-25',
				category: 'Applications',
				uploader: 'ubuntu'
			},
			{
				id: '2',
				name: 'Fedora Workstation 40 x86_64',
				size: '2.1 GB',
				seeders: 580,
				leechers: 22,
				magnetLink: 'magnet:?xt=urn:btih:def456',
				infoHash: 'def456',
				uploadDate: '2024-05-01',
				category: 'Applications',
				uploader: 'fedora'
			}
		],
		sort: { field: 'seeders', direction: 'desc' },
		addingTorrents: new Set(),
		disableAdd: false
	}
};

export const Empty: Story = {
	args: {
		results: [],
		sort: { field: 'seeders', direction: 'desc' },
		addingTorrents: new Set(),
		disableAdd: false
	}
};
