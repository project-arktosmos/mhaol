import type { Meta, StoryObj } from '@storybook/svelte';
import MediaListCard from 'ui-lib/components/media/MediaListCard.svelte';

const meta = {
	title: 'Media/MediaListCard',
	component: MediaListCard,
	tags: ['autodocs']
} satisfies Meta<typeof MediaListCard>;

export default meta;
type Story = StoryObj<typeof meta>;

export const TvShowSeason: Story = {
	args: {
		list: {
			id: 'list-1',
			libraryId: 'lib-1',
			title: 'Breaking Bad - Season 1',
			description: 'Season 1 of Breaking Bad',
			coverImage: null,
			mediaType: 'video',
			libraryType: 'local',
			source: 'auto',
			itemCount: 7,
			createdAt: '2024-01-15T10:30:00Z',
			links: {
				tmdb: { serviceId: '1396', seasonNumber: 1 }
			},
			items: []
		}
	}
};

export const MusicAlbum: Story = {
	args: {
		list: {
			id: 'list-2',
			libraryId: 'lib-1',
			title: 'A Night at the Opera',
			description: 'Queen album',
			coverImage: 'https://picsum.photos/seed/queen-album/300/300',
			mediaType: 'audio',
			libraryType: 'local',
			source: 'auto',
			itemCount: 12,
			createdAt: '2024-01-15T10:30:00Z',
			links: {},
			items: []
		}
	}
};

export const Selected: Story = {
	args: {
		list: {
			id: 'list-3',
			libraryId: 'lib-1',
			title: 'The Office - Season 3',
			description: null,
			coverImage: null,
			mediaType: 'video',
			libraryType: 'local',
			source: 'auto',
			itemCount: 23,
			createdAt: '2024-01-15T10:30:00Z',
			links: {},
			items: []
		},
		selected: true
	}
};

export const NoCover: Story = {
	args: {
		list: {
			id: 'list-4',
			libraryId: 'lib-1',
			title: 'Untitled Collection',
			description: null,
			coverImage: null,
			mediaType: 'video',
			libraryType: 'local',
			source: 'user',
			itemCount: 3,
			createdAt: '2024-01-15T10:30:00Z',
			links: {},
			items: []
		}
	}
};
