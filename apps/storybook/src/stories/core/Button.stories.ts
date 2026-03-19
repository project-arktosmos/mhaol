import type { Meta, StoryObj } from '@storybook/svelte';
import Button from 'ui-lib/components/core/Button.svelte';

const meta = {
	title: 'Core/Button',
	component: Button,
	tags: ['autodocs'],
	argTypes: {
		color: {
			control: 'select',
			options: ['primary', 'secondary', 'accent', 'success', 'error', 'info', 'warning', 'neutral']
		},
		size: { control: 'select', options: ['xs', 'sm', 'md', 'lg', 'xl'] },
		outline: { control: 'boolean' },
		disabled: { control: 'boolean' },
		wide: { control: 'boolean' },
		tall: { control: 'boolean' },
		label: { control: 'text' }
	}
} satisfies Meta<typeof Button>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Primary: Story = { args: { label: 'Click me', color: 'primary' } };
export const Secondary: Story = { args: { label: 'Secondary', color: 'secondary' } };
export const Accent: Story = { args: { label: 'Accent', color: 'accent' } };
export const Success: Story = { args: { label: 'Success', color: 'success' } };
export const Error: Story = { args: { label: 'Error', color: 'error' } };
export const Outline: Story = { args: { label: 'Outlined', color: 'primary', outline: true } };
export const Small: Story = { args: { label: 'Small', size: 'sm' } };
export const Large: Story = { args: { label: 'Large', size: 'lg' } };
export const Wide: Story = { args: { label: 'Full width', wide: true } };
export const Disabled: Story = { args: { label: 'Disabled', disabled: true } };
export const Link: Story = { args: { label: 'Visit site', href: 'https://example.com', target: '_blank' } };
