import type { Meta, StoryObj } from '@storybook/svelte';
import Navbar from 'ui-lib/components/core/Navbar.svelte';

const meta = {
	title: 'Core/Navbar',
	component: Navbar,
	tags: ['autodocs']
} satisfies Meta<typeof Navbar>;

export default meta;
type Story = StoryObj<typeof meta>;

export const WithItems: Story = {
	args: {
		brand: { label: 'Mhaol', highlight: 'Tube' },
		items: [
			{ id: 'demo1', label: 'Feature A', classes: 'btn-primary' },
			{ id: 'demo2', label: 'Feature B', classes: 'btn-secondary' }
		]
	}
};

export const Minimal: Story = { args: { brand: { label: 'Stories' }, items: [] } };
