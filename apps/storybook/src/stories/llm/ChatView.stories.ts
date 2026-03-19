import type { Meta, StoryObj } from '@storybook/svelte';
import ChatView from 'ui-lib/components/llm/ChatView.svelte';

const meta = {
	title: 'LLM/ChatView',
	component: ChatView,
	tags: ['autodocs']
} satisfies Meta<typeof ChatView>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Empty: Story = {
	args: {
		messages: [],
		streamingContent: '',
		isGenerating: false,
		onSendMessage: (content: string) => console.log('Send:', content),
		onCancelGeneration: () => console.log('Cancel')
	}
};

export const WithConversation: Story = {
	args: {
		messages: [
			{ role: 'user', content: 'What is the capital of France?' },
			{
				role: 'assistant',
				content:
					'The capital of France is Paris. It is the largest city in France and serves as the country\'s political, economic, and cultural center.'
			},
			{ role: 'user', content: 'What about Germany?' },
			{
				role: 'assistant',
				content:
					'The capital of Germany is Berlin. It is the largest city in Germany by both area and population.'
			}
		],
		streamingContent: '',
		isGenerating: false,
		onSendMessage: (content: string) => console.log('Send:', content),
		onCancelGeneration: () => console.log('Cancel')
	}
};

export const StreamingResponse: Story = {
	args: {
		messages: [
			{ role: 'user', content: 'Tell me about quantum computing' }
		],
		streamingContent: 'Quantum computing is a type of computation that harnesses quantum mechanical phenomena such as',
		isGenerating: true,
		onSendMessage: (content: string) => console.log('Send:', content),
		onCancelGeneration: () => console.log('Cancel')
	}
};

export const Generating: Story = {
	args: {
		messages: [
			{ role: 'user', content: 'Write a haiku about programming' }
		],
		streamingContent: '',
		isGenerating: true,
		onSendMessage: (content: string) => console.log('Send:', content),
		onCancelGeneration: () => console.log('Cancel')
	}
};
