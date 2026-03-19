import type { Meta, StoryObj } from '@storybook/svelte';
import TorrentModalContent from 'ui-lib/components/torrent/TorrentModalContent.svelte';

const meta = {
	title: 'Torrent/TorrentModalContent',
	component: TorrentModalContent,
	tags: ['autodocs']
} satisfies Meta<typeof TorrentModalContent>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
