import type { Meta, StoryObj } from '@storybook/svelte';
import TorrentList from 'ui-lib/components/torrent/TorrentList.svelte';

const meta = {
	title: 'Torrent/TorrentList',
	component: TorrentList,
	tags: ['autodocs']
} satisfies Meta<typeof TorrentList>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
