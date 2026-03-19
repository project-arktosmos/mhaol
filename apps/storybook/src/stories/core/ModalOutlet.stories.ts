import type { Meta, StoryObj } from '@storybook/svelte';
import ModalOutlet from 'ui-lib/components/core/ModalOutlet.svelte';

const meta = {
	title: 'Core/ModalOutlet',
	component: ModalOutlet,
	tags: ['autodocs']
} satisfies Meta<typeof ModalOutlet>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = { args: { modals: {} } };
