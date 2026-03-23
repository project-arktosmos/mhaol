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

export interface LlmState {
	status: LlmStatus | null;
	models: LocalModel[];
	downloadProgress: LlmDownloadProgress | null;
	loading: boolean;
}
