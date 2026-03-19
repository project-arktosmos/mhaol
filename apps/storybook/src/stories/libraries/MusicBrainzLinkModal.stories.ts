import type { Meta, StoryObj } from '@storybook/svelte';
import MusicBrainzLinkModal from 'ui-lib/components/libraries/MusicBrainzLinkModal.svelte';

const meta = {
	title: 'Libraries/MusicBrainzLinkModal',
	component: MusicBrainzLinkModal,
	tags: ['autodocs'],
	args: {
		file: { id: 1, libraryId: 1, path: '/media/song.mp3', name: 'song.mp3', size: 524288, mediaType: 'audio' },
		onlink: () => {},
		onclose: () => {}
	}
} satisfies Meta<typeof MusicBrainzLinkModal>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
