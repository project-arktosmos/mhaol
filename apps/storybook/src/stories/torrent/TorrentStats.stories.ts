import type { Meta, StoryObj } from '@storybook/svelte';
import TorrentStats from 'ui-lib/components/torrent/TorrentStats.svelte';

const meta = {
	title: 'Torrent/TorrentStats',
	component: TorrentStats,
	tags: ['autodocs']
} satisfies Meta<typeof TorrentStats>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
