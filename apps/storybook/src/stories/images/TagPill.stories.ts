import type { Meta, StoryObj } from '@storybook/svelte';
import TagPill from 'ui-lib/components/images/TagPill.svelte';

const meta = {
	title: 'Images/TagPill',
	component: TagPill,
	tags: ['autodocs'],
	argTypes: {
		tag: { control: 'text' },
		score: { control: { type: 'range', min: 0, max: 1, step: 0.01 } },
		readonly: { control: 'boolean' }
	}
} satisfies Meta<typeof TagPill>;

export default meta;
type Story = StoryObj<typeof meta>;

export const HighConfidence: Story = { args: { tag: 'landscape', score: 0.95 } };
export const MediumConfidence: Story = { args: { tag: 'portrait', score: 0.05 } };
export const LowConfidence: Story = { args: { tag: 'abstract', score: 0.01 } };
export const Readonly: Story = { args: { tag: 'nature', score: 0.8, readonly: true } };
