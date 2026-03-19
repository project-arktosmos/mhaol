import type { Meta, StoryObj } from '@storybook/svelte';
import TorrentSettings from 'ui-lib/components/torrent/TorrentSettings.svelte';

const meta = {
	title: 'Torrent/TorrentSettings',
	component: TorrentSettings,
	tags: ['autodocs']
} satisfies Meta<typeof TorrentSettings>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
