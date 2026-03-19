import type { Meta, StoryObj } from '@storybook/svelte';
import TorrentAddInput from 'ui-lib/components/torrent/TorrentAddInput.svelte';

const meta = {
	title: 'Torrent/TorrentAddInput',
	component: TorrentAddInput,
	tags: ['autodocs']
} satisfies Meta<typeof TorrentAddInput>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
