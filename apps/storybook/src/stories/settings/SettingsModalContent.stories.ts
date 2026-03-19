import type { Meta, StoryObj } from '@storybook/svelte';
import SettingsModalContent from 'ui-lib/components/settings/SettingsModalContent.svelte';

const meta = {
	title: 'Settings/SettingsModalContent',
	component: SettingsModalContent,
	tags: ['autodocs']
} satisfies Meta<typeof SettingsModalContent>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
