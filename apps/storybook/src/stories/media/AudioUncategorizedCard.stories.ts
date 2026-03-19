import type { Meta, StoryObj } from '@storybook/svelte';
import AudioUncategorizedCard from 'ui-lib/components/media/AudioUncategorizedCard.svelte';

const meta = {
	title: 'Media/AudioUncategorizedCard',
	component: AudioUncategorizedCard,
	tags: ['autodocs']
} satisfies Meta<typeof AudioUncategorizedCard>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
	args: {
		item: {
			id: '1',
			libraryId: 'lib-1',
			name: 'Voice Memo.mp3',
			extension: '.mp3',
			path: '/media/audio/Voice Memo.mp3',
			categoryId: null,
			mediaTypeId: 'audio',
			createdAt: '2024-01-15T10:30:00Z',
			links: {}
		}
	}
};

export const WithMetadata: Story = {
	args: {
		item: {
			id: '2',
			libraryId: 'lib-1',
			name: 'Bohemian Rhapsody.flac',
			extension: '.flac',
			path: '/media/audio/Bohemian Rhapsody.flac',
			categoryId: null,
			mediaTypeId: 'audio',
			createdAt: '2024-01-15T10:30:00Z',
			links: {
				musicbrainz: { serviceId: 'mb-123', seasonNumber: null, episodeNumber: null }
			}
		},
		metadata: {
			title: 'Bohemian Rhapsody',
			artistCredits: 'Queen',
			coverArtUrl: 'https://picsum.photos/seed/bohemian/300/300',
			duration: '5:55'
		}
	}
};

export const Selected: Story = {
	args: {
		item: {
			id: '3',
			libraryId: 'lib-1',
			name: 'Podcast Episode.mp3',
			extension: '.mp3',
			path: '/media/audio/Podcast Episode.mp3',
			categoryId: null,
			mediaTypeId: 'audio',
			createdAt: '2024-01-15T10:30:00Z',
			links: {}
		},
		selected: true
	}
};
