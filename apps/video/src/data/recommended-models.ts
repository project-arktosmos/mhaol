import type { RecommendedModel } from '$types/llm.type';

export const recommendedModels: RecommendedModel[] = [
	{
		name: 'Qwen2.5 0.5B',
		repoId: 'Qwen/Qwen2.5-0.5B-Instruct-GGUF',
		fileName: 'qwen2.5-0.5b-instruct-q4_k_m.gguf',
		sizeGb: 0.4,
		description: 'Tiny model, fast inference (~400 MB)'
	},
	{
		name: 'Qwen2.5 1.5B',
		repoId: 'Qwen/Qwen2.5-1.5B-Instruct-GGUF',
		fileName: 'qwen2.5-1.5b-instruct-q4_k_m.gguf',
		sizeGb: 1.0,
		description: 'Small model, good quality (~1.0 GB)'
	},
	{
		name: 'Qwen2.5 3B',
		repoId: 'Qwen/Qwen2.5-3B-Instruct-GGUF',
		fileName: 'qwen2.5-3b-instruct-q4_k_m.gguf',
		sizeGb: 2.0,
		description: 'Medium model, better quality (~2.0 GB)'
	},
	{
		name: 'SmolLM2 1.7B',
		repoId: 'HuggingFaceTB/SmolLM2-1.7B-Instruct-GGUF',
		fileName: 'smollm2-1.7b-instruct-q4_k_m.gguf',
		sizeGb: 1.0,
		description: 'Compact model by HuggingFace (~1.0 GB)'
	}
];
