import { AdapterClass } from '$adapters/classes/adapter.class';

export class LlmAdapter extends AdapterClass {
	constructor() {
		super('llm');
	}

	formatModelSize(bytes: number): string {
		if (bytes >= 1_073_741_824) {
			return `${(bytes / 1_073_741_824).toFixed(1)} GB`;
		}
		if (bytes >= 1_048_576) {
			return `${(bytes / 1_048_576).toFixed(0)} MB`;
		}
		return `${(bytes / 1024).toFixed(0)} KB`;
	}
}

export const llmAdapter = new LlmAdapter();
