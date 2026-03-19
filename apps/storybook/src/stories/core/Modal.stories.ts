import type { Meta, StoryObj } from '@storybook/svelte';
import Modal from 'ui-lib/components/core/Modal.svelte';

const meta = {
	title: 'Core/Modal',
	component: Modal,
	tags: ['autodocs'],
	argTypes: {
		open: { control: 'boolean' },
		maxWidth: { control: 'select', options: ['max-w-sm', 'max-w-lg', 'max-w-2xl', 'max-w-4xl', 'max-w-5xl'] }
	}
} satisfies Meta<typeof Modal>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = { args: { open: true, maxWidth: 'max-w-lg' } };
export const Wide: Story = { args: { open: true, maxWidth: 'max-w-4xl' } };
