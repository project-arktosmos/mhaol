import type { Meta, StoryObj } from '@storybook/svelte';
import ImageUncategorizedCard from 'ui-lib/components/media/ImageUncategorizedCard.svelte';

const meta = {
	title: 'Media/ImageUncategorizedCard',
	component: ImageUncategorizedCard,
	tags: ['autodocs']
} satisfies Meta<typeof ImageUncategorizedCard>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
	args: {
		item: {
			id: '1',
			libraryId: 'lib-1',
			name: 'Sunset Photo.jpg',
			extension: '.jpg',
			path: '/media/images/Sunset Photo.jpg',
			categoryId: null,
			mediaTypeId: 'image',
			createdAt: '2024-01-15T10:30:00Z',
			links: {}
		}
	}
};

export const WithTags: Story = {
	args: {
		item: {
			id: '2',
			libraryId: 'lib-1',
			name: 'Mountain Landscape.png',
			extension: '.png',
			path: '/media/images/Mountain Landscape.png',
			categoryId: null,
			mediaTypeId: 'image',
			createdAt: '2024-01-15T10:30:00Z',
			links: {}
		},
		tags: [
			{ tag: 'landscape', score: 0.95 },
			{ tag: 'mountain', score: 0.88 },
			{ tag: 'nature', score: 0.82 }
		]
	}
};

export const Tagging: Story = {
	args: {
		item: {
			id: '3',
			libraryId: 'lib-1',
			name: 'Processing.jpg',
			extension: '.jpg',
			path: '/media/images/Processing.jpg',
			categoryId: null,
			mediaTypeId: 'image',
			createdAt: '2024-01-15T10:30:00Z',
			links: {}
		},
		tagging: true
	}
};
