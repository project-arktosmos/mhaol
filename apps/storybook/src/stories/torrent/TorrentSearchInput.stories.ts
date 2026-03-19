import type { Meta, StoryObj } from '@storybook/svelte';
import TorrentSearchInput from 'ui-lib/components/torrent/TorrentSearchInput.svelte';

const meta = {
	title: 'Torrent/TorrentSearchInput',
	component: TorrentSearchInput,
	tags: ['autodocs'],
	argTypes: {
		query: { control: 'text' },
		category: { control: 'text' },
		searching: { control: 'boolean' }
	}
} satisfies Meta<typeof TorrentSearchInput>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = { args: { query: '', searching: false } };
export const WithQuery: Story = { args: { query: 'ubuntu iso', searching: false } };
export const Searching: Story = { args: { query: 'ubuntu iso', searching: true } };
