import type { Meta, StoryObj } from '@storybook/svelte';
import NavbarLink from 'ui-lib/components/core/NavbarLink.svelte';

const meta = {
	title: 'Core/NavbarLink',
	component: NavbarLink,
	tags: ['autodocs']
} satisfies Meta<typeof NavbarLink>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = { args: { href: '/', label: 'Home', currentPath: '/other' } };
export const Active: Story = { args: { href: '/', label: 'Home', currentPath: '/' } };
