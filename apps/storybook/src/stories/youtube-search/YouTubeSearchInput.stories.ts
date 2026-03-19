import type { Meta, StoryObj } from '@storybook/svelte';
import YouTubeSearchInput from 'ui-lib/components/youtube-search/YouTubeSearchInput.svelte';

const meta = {
	title: 'YouTubeSearch/YouTubeSearchInput',
	component: YouTubeSearchInput,
	tags: ['autodocs'],
	argTypes: {
		query: { control: 'text' },
		searching: { control: 'boolean' }
	}
} satisfies Meta<typeof YouTubeSearchInput>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = { args: { query: '', searching: false, onsearch: (q: string) => console.log('search:', q) } };
export const WithQuery: Story = { args: { query: 'lofi hip hop', searching: false, onsearch: (q: string) => console.log('search:', q) } };
export const Searching: Story = { args: { query: 'lofi hip hop', searching: true, onsearch: (q: string) => console.log('search:', q) } };
