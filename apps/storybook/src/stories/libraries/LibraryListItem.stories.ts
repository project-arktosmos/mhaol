import type { Meta, StoryObj } from '@storybook/svelte';
import LibraryListItem from 'ui-lib/components/libraries/LibraryListItem.svelte';

const meta = {
	title: 'Libraries/LibraryListItem',
	component: LibraryListItem,
	tags: ['autodocs'],
	args: {
		library: { id: 1, name: 'Movies', path: '/media/movies', mediaType: 'video' },
		files: [
			{ id: 1, libraryId: 1, path: '/media/movies/movie.mp4', name: 'movie.mp4', size: 1048576, mediaType: 'video' }
		],
		filesLoading: false,
		filesError: ''
	}
} satisfies Meta<typeof LibraryListItem>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
