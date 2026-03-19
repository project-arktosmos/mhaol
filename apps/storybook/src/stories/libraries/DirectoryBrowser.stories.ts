import type { Meta, StoryObj } from '@storybook/svelte';
import DirectoryBrowser from 'ui-lib/components/libraries/DirectoryBrowser.svelte';

const meta = {
	title: 'Libraries/DirectoryBrowser',
	component: DirectoryBrowser,
	tags: ['autodocs'],
	args: {
		onselect: () => {}
	}
} satisfies Meta<typeof DirectoryBrowser>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
