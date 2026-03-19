import type { Meta, StoryObj } from '@storybook/svelte';
import TorrentSearch from 'ui-lib/components/torrent/TorrentSearch.svelte';

const meta = {
	title: 'Torrent/TorrentSearch',
	component: TorrentSearch,
	tags: ['autodocs']
} satisfies Meta<typeof TorrentSearch>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
