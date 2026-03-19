import type { Meta, StoryObj } from '@storybook/svelte';
import ConversationList from 'ui-lib/components/llm/ConversationList.svelte';

const meta = {
	title: 'LLM/ConversationList',
	component: ConversationList,
	tags: ['autodocs']
} satisfies Meta<typeof ConversationList>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Empty: Story = {
	args: {
		conversations: [],
		activeId: null,
		onSelect: (id: string) => console.log('Select:', id),
		onDelete: (id: string) => console.log('Delete:', id),
		onCreate: () => console.log('Create')
	}
};

export const WithConversations: Story = {
	args: {
		conversations: [
			{
				id: 'conv-1',
				title: 'Chat about AI',
				systemPrompt: null,
				messages: '[]',
				createdAt: '2024-01-15T10:30:00Z',
				updatedAt: '2024-01-15T11:00:00Z'
			},
			{
				id: 'conv-2',
				title: 'Recipe ideas',
				systemPrompt: null,
				messages: '[]',
				createdAt: '2024-01-14T09:00:00Z',
				updatedAt: '2024-01-14T09:30:00Z'
			},
			{
				id: 'conv-3',
				title: 'Code review help',
				systemPrompt: null,
				messages: '[]',
				createdAt: '2024-01-13T14:00:00Z',
				updatedAt: '2024-01-13T15:00:00Z'
			}
		],
		activeId: 'conv-1',
		onSelect: (id: string) => console.log('Select:', id),
		onDelete: (id: string) => console.log('Delete:', id),
		onCreate: () => console.log('Create')
	}
};

export const NoActiveSelection: Story = {
	args: {
		conversations: [
			{
				id: 'conv-1',
				title: 'First conversation',
				systemPrompt: null,
				messages: '[]',
				createdAt: '2024-01-15T10:30:00Z',
				updatedAt: '2024-01-15T11:00:00Z'
			},
			{
				id: 'conv-2',
				title: 'Second conversation',
				systemPrompt: null,
				messages: '[]',
				createdAt: '2024-01-14T09:00:00Z',
				updatedAt: '2024-01-14T09:30:00Z'
			}
		],
		activeId: null,
		onSelect: (id: string) => console.log('Select:', id),
		onDelete: (id: string) => console.log('Delete:', id),
		onCreate: () => console.log('Create')
	}
};
