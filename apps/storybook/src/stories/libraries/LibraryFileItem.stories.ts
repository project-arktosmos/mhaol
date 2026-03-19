import type { Meta, StoryObj } from '@storybook/svelte';
import LibraryFileItem from 'ui-lib/components/libraries/LibraryFileItem.svelte';

const meta = {
	title: 'Libraries/LibraryFileItem',
	component: LibraryFileItem,
	tags: ['autodocs'],
	args: {
		file: {
			id: 1,
			libraryId: 1,
			path: '/media/file.mp4',
			name: 'file.mp4',
			size: 1048576,
			mediaType: 'video'
		},
		onplay: () => {},
		ondelete: () => {},
		oncategorize: () => {},
		onlink: () => {},
		onstream: () => {}
	}
} satisfies Meta<typeof LibraryFileItem>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
