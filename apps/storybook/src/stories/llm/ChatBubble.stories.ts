import type { Meta, StoryObj } from '@storybook/svelte';
import ChatBubble from 'ui-lib/components/llm/ChatBubble.svelte';

const meta = {
	title: 'LLM/ChatBubble',
	component: ChatBubble,
	tags: ['autodocs']
} satisfies Meta<typeof ChatBubble>;

export default meta;
type Story = StoryObj<typeof meta>;

export const UserMessage: Story = {
	args: {
		message: {
			role: 'user',
			content: 'Can you explain how neural networks work?'
		},
		isStreaming: false
	}
};

export const AssistantMessage: Story = {
	args: {
		message: {
			role: 'assistant',
			content:
				'Neural networks are computing systems inspired by biological neural networks. They consist of layers of interconnected nodes (neurons) that process information. Each connection has a weight that adjusts during training, allowing the network to learn patterns from data.'
		},
		isStreaming: false
	}
};

export const SystemMessage: Story = {
	args: {
		message: {
			role: 'system',
			content: 'You are a helpful assistant that explains complex topics simply.'
		},
		isStreaming: false
	}
};

export const Streaming: Story = {
	args: {
		message: {
			role: 'assistant',
			content: 'Neural networks are computing systems inspired by'
		},
		isStreaming: true
	}
};

export const LongMessage: Story = {
	args: {
		message: {
			role: 'assistant',
			content:
				'Here is a detailed explanation:\n\n1. Input Layer: Receives the raw data\n2. Hidden Layers: Process the data through weighted connections\n3. Output Layer: Produces the final result\n\nEach neuron applies an activation function to determine whether it should fire or not. Common activation functions include ReLU, sigmoid, and tanh.\n\nThe training process uses backpropagation to adjust weights based on the error between predicted and actual outputs.'
		},
		isStreaming: false
	}
};
