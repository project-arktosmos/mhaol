import type { Meta, StoryObj } from '@storybook/svelte';
import SeasonEpisodeMatchModal from 'ui-lib/components/libraries/SeasonEpisodeMatchModal.svelte';

const meta = {
	title: 'Libraries/SeasonEpisodeMatchModal',
	component: SeasonEpisodeMatchModal,
	tags: ['autodocs'],
	args: {
		tmdbId: 12345,
		seasonNumber: 1,
		showName: 'Sample Show',
		seasonName: 'Season 1',
		files: [
			{ id: 1, libraryId: 1, path: '/media/s01e01.mp4', name: 's01e01.mp4', size: 1048576, mediaType: 'video' },
			{ id: 2, libraryId: 1, path: '/media/s01e02.mp4', name: 's01e02.mp4', size: 1048576, mediaType: 'video' }
		],
		onlinkall: () => {},
		onclose: () => {}
	}
} satisfies Meta<typeof SeasonEpisodeMatchModal>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
