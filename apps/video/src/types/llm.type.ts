export interface LlmStatus {
	available: boolean;
	modelLoaded: boolean;
	currentModel: string | null;
	temperature: number;
	topP: number;
	topK: number;
	repeatPenalty: number;
	maxTokens: number;
	systemPrompt: string;
	modelsDir: string;
	error: string | null;
}

export interface LocalModel {
	name: string;
	fileName: string;
	sizeBytes: number;
	path: string;
	isLoaded: boolean;
}

export interface ChatMessage {
	role: 'system' | 'user' | 'assistant';
	content: string;
}

export interface LlmConversation {
	id: string;
	title: string;
	systemPrompt: string | null;
	messages: string;
	createdAt: string;
	updatedAt: string;
}

export interface RecommendedModel {
	repoId: string;
	fileName: string;
	name: string;
	sizeGb: number;
	description: string;
}

export interface LlmConfigUpdate {
	temperature?: number;
	topP?: number;
	topK?: number;
	repeatPenalty?: number;
	maxTokens?: number;
	systemPrompt?: string;
}

export interface LlmDownloadProgress {
	modelName: string;
	downloadedBytes: number;
	totalBytes: number;
	percent: number;
	status: string;
}

export interface LlmTokenEvent {
	content: string;
	done: boolean;
}

export interface LlmState {
	status: LlmStatus | null;
	models: LocalModel[];
	conversations: LlmConversation[];
	activeConversationId: string | null;
	messages: ChatMessage[];
	streamingContent: string;
	isGenerating: boolean;
	downloadProgress: LlmDownloadProgress | null;
	loading: boolean;
}
