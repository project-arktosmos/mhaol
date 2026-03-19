import type { Meta, StoryObj } from '@storybook/svelte';
import LlmModalContent from 'ui-lib/components/llm/LlmModalContent.svelte';

const meta = {
	title: 'LLM/LlmModalContent',
	component: LlmModalContent,
	tags: ['autodocs']
} satisfies Meta<typeof LlmModalContent>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
