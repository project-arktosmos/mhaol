import type { Meta, StoryObj } from '@storybook/svelte';
import ModelManager from 'ui-lib/components/llm/ModelManager.svelte';

const meta = {
	title: 'LLM/ModelManager',
	component: ModelManager,
	tags: ['autodocs']
} satisfies Meta<typeof ModelManager>;

export default meta;
type Story = StoryObj<typeof meta>;

export const NoModels: Story = {
	args: {
		status: {
			available: true,
			modelLoaded: false,
			currentModel: null,
			temperature: 0.7,
			topP: 0.9,
			topK: 40,
			repeatPenalty: 1.1,
			maxTokens: 2048,
			systemPrompt: 'You are a helpful assistant.',
			modelsDir: '/home/user/.local/share/mhaol/models',
			error: null
		},
		models: [],
		downloadProgress: null,
		loading: false,
		onLoadModel: (fileName: string) => console.log('Load:', fileName),
		onUnloadModel: () => console.log('Unload'),
		onDownloadModel: (repoId: string, fileName: string) => console.log('Download:', repoId, fileName)
	}
};

export const WithModels: Story = {
	args: {
		status: {
			available: true,
			modelLoaded: true,
			currentModel: 'mistral-7b-instruct-v0.2.Q4_K_M.gguf',
			temperature: 0.7,
			topP: 0.9,
			topK: 40,
			repeatPenalty: 1.1,
			maxTokens: 2048,
			systemPrompt: 'You are a helpful assistant.',
			modelsDir: '/home/user/.local/share/mhaol/models',
			error: null
		},
		models: [
			{
				name: 'Mistral 7B Instruct v0.2',
				fileName: 'mistral-7b-instruct-v0.2.Q4_K_M.gguf',
				sizeBytes: 4368438272,
				path: '/home/user/.local/share/mhaol/models/mistral-7b-instruct-v0.2.Q4_K_M.gguf',
				isLoaded: true
			},
			{
				name: 'Llama 2 7B Chat',
				fileName: 'llama-2-7b-chat.Q4_K_M.gguf',
				sizeBytes: 4081004544,
				path: '/home/user/.local/share/mhaol/models/llama-2-7b-chat.Q4_K_M.gguf',
				isLoaded: false
			}
		],
		downloadProgress: null,
		loading: false,
		onLoadModel: (fileName: string) => console.log('Load:', fileName),
		onUnloadModel: () => console.log('Unload'),
		onDownloadModel: (repoId: string, fileName: string) => console.log('Download:', repoId, fileName)
	}
};

export const Downloading: Story = {
	args: {
		status: {
			available: true,
			modelLoaded: false,
			currentModel: null,
			temperature: 0.7,
			topP: 0.9,
			topK: 40,
			repeatPenalty: 1.1,
			maxTokens: 2048,
			systemPrompt: 'You are a helpful assistant.',
			modelsDir: '/home/user/.local/share/mhaol/models',
			error: null
		},
		models: [],
		downloadProgress: {
			modelName: 'Mistral 7B Instruct v0.2',
			downloadedBytes: 1747375308,
			totalBytes: 4368438272,
			percent: 40.0,
			status: 'downloading'
		},
		loading: false,
		onLoadModel: (fileName: string) => console.log('Load:', fileName),
		onUnloadModel: () => console.log('Unload'),
		onDownloadModel: (repoId: string, fileName: string) => console.log('Download:', repoId, fileName)
	}
};

export const LoadingModel: Story = {
	args: {
		status: {
			available: true,
			modelLoaded: false,
			currentModel: null,
			temperature: 0.7,
			topP: 0.9,
			topK: 40,
			repeatPenalty: 1.1,
			maxTokens: 2048,
			systemPrompt: 'You are a helpful assistant.',
			modelsDir: '/home/user/.local/share/mhaol/models',
			error: null
		},
		models: [
			{
				name: 'Mistral 7B Instruct v0.2',
				fileName: 'mistral-7b-instruct-v0.2.Q4_K_M.gguf',
				sizeBytes: 4368438272,
				path: '/home/user/.local/share/mhaol/models/mistral-7b-instruct-v0.2.Q4_K_M.gguf',
				isLoaded: false
			}
		],
		downloadProgress: null,
		loading: true,
		onLoadModel: (fileName: string) => console.log('Load:', fileName),
		onUnloadModel: () => console.log('Unload'),
		onDownloadModel: (repoId: string, fileName: string) => console.log('Download:', repoId, fileName)
	}
};
