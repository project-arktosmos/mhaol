import type { Meta, StoryObj } from '@storybook/svelte';
import PluginsModalContent from 'ui-lib/components/plugins/PluginsModalContent.svelte';

const meta = {
	title: 'Plugins/PluginsModalContent',
	component: PluginsModalContent,
	tags: ['autodocs']
} satisfies Meta<typeof PluginsModalContent>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
