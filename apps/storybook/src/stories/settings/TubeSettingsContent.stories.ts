import type { Meta, StoryObj } from '@storybook/svelte';
import TubeSettingsContent from 'ui-lib/components/settings/TubeSettingsContent.svelte';

const meta = {
	title: 'Settings/TubeSettingsContent',
	component: TubeSettingsContent,
	tags: ['autodocs']
} satisfies Meta<typeof TubeSettingsContent>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
