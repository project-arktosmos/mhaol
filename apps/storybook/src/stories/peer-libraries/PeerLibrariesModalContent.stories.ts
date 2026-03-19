import type { Meta, StoryObj } from '@storybook/svelte';
import PeerLibrariesModalContent from 'ui-lib/components/peer-libraries/PeerLibrariesModalContent.svelte';

const meta = {
	title: 'PeerLibraries/PeerLibrariesModalContent',
	component: PeerLibrariesModalContent,
	tags: ['autodocs']
} satisfies Meta<typeof PeerLibrariesModalContent>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
