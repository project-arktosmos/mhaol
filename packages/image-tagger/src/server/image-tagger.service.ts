import { pipeline, RawImage, type ZeroShotImageClassificationPipeline } from '@huggingface/transformers';
import { readFile } from 'node:fs/promises';
import type { TagResult, TaggerProgress } from '../shared/types.js';

// Labels are full natural-language phrases so CLIP gets well-formed sentences.
// The display tag (shown to the user) is derived by stripping the "a photo of " prefix.
const DEFAULT_LABELS = [
	'a photo of a person',
	'a photo of a face',
	'a photo of a man',
	'a photo of a woman',
	'a photo of a child',
	'a photo of people',
	'a portrait photo',
	'a selfie',
	'a photo of a group of people',
	'a photo of a baby',
	'a family photo',
	'a photo of a dog',
	'a photo of a cat',
	'a photo of a bird',
	'a photo of an animal',
	'a photo of a pet',
	'a photo of a beach',
	'a photo of a mountain',
	'a photo of a city',
	'a photo of a forest',
	'a photo of a park',
	'a photo of a garden',
	'an indoor photo',
	'an outdoor photo',
	'a photo of a street',
	'a photo of a building',
	'a photo of food',
	'a photo of cooking',
	'a travel photo',
	'a photo of sports',
	'a photo of a concert',
	'a photo of a party',
	'a photo of a wedding',
	'a photo of a graduation',
	'a photo of a birthday',
	'a photo of a car',
	'a photo of a bicycle',
	'a photo of a flower',
	'a photo of a book',
	'a photo of a computer',
	'a photo of a phone',
	'a screenshot',
	'a photo of a document',
	'a photo of a receipt',
	'a photo of a whiteboard',
	'a photo of handwriting',
	'a meme',
	'a photo of a chart',
	'a photo of a map',
	'a photo of a sunset',
	'a photo of a sunrise',
	'a photo of snow',
	'a photo of rain',
	'a photo of the night sky',
	'a photo of the ocean',
	'a photo of artwork',
	'a painting',
	'a black and white photo',
	'an aerial view',
	'a macro photo'
];

// Map from label → display tag
function labelToTag(label: string): string {
	return label
		.replace(/^an? photo of (an?|the) /, '')
		.replace(/^an? photo of /, '')
		.replace(/^an? /, '')
		.replace(/ photo$/, '');
}

const DEFAULT_TAGS = DEFAULT_LABELS.map(labelToTag);

const DEFAULT_THRESHOLD = 0.005;

let classifierPromise: Promise<ZeroShotImageClassificationPipeline> | null = null;
let ready = false;

const progress: TaggerProgress = {
	status: 'idle',
	files: {},
	overallProgress: 0,
	error: null
};

function updateOverallProgress() {
	const entries = Object.values(progress.files);
	if (entries.length === 0) {
		progress.overallProgress = 0;
		return;
	}
	const totalBytes = entries.reduce((sum, f) => sum + f.total, 0);
	const loadedBytes = entries.reduce((sum, f) => sum + f.loaded, 0);
	progress.overallProgress = totalBytes > 0 ? Math.round((loadedBytes / totalBytes) * 100) : 0;
}

function getClassifier(): Promise<ZeroShotImageClassificationPipeline> {
	if (!classifierPromise) {
		console.log('[image-tagger] Initializing SigLIP pipeline (first use, downloading model)...');
		progress.status = 'downloading';
		progress.error = null;

		classifierPromise = pipeline(
			'zero-shot-image-classification',
			'Xenova/siglip-large-patch16-384',
			{
				dtype: 'fp32',
				progress_callback: (event: {
					status: string;
					file?: string;
					progress?: number;
					loaded?: number;
					total?: number;
				}) => {
					if (event.status === 'progress' && event.file) {
						progress.files[event.file] = {
							loaded: event.loaded ?? 0,
							total: event.total ?? 0,
							progress: event.progress ?? 0
						};
						progress.status = 'downloading';
						updateOverallProgress();
					} else if (event.status === 'done' && event.file) {
						if (progress.files[event.file]) {
							progress.files[event.file].progress = 100;
							progress.files[event.file].loaded = progress.files[event.file].total;
						}
						updateOverallProgress();
					} else if (event.status === 'ready') {
						progress.status = 'loading';
						progress.overallProgress = 100;
					}
				}
			}
		)
			.then((p) => {
				ready = true;
				progress.status = 'ready';
				progress.overallProgress = 100;
				console.log('[image-tagger] SigLIP pipeline ready');
				return p;
			})
			.catch((err) => {
				progress.status = 'error';
				progress.error = err instanceof Error ? err.message : String(err);
				classifierPromise = null;
				throw err;
			});
	}
	return classifierPromise;
}

function softmax(logits: Float32Array | number[]): number[] {
	const max = Math.max(...logits);
	const exps = Array.from(logits, (x) => Math.exp(x - max));
	const sum = exps.reduce((a, b) => a + b, 0);
	return exps.map((x) => x / sum);
}

export async function tagImage(
	imagePath: string,
	labels: string[] = DEFAULT_LABELS,
	threshold: number = DEFAULT_THRESHOLD
): Promise<TagResult[]> {
	const classifier = await getClassifier();
	const imageBuffer = await readFile(imagePath);
	const blob = new Blob([imageBuffer]);
	const url = URL.createObjectURL(blob);

	try {
		// Use model directly to get raw logits, then apply softmax for interpretable scores.
		// SigLIP's raw sigmoid scores are near-zero for zero-shot tasks, so softmax
		// gives much more useful, thresholdable probabilities.
		const image = await RawImage.fromURL(url);
		const imageInputs = await classifier.processor(image);
		const textInputs = classifier.tokenizer(labels, { padding: 'max_length', truncation: true });
		const output = await classifier.model({ ...textInputs, pixel_values: imageInputs.pixel_values });

		const logits = output.logits_per_image.data as Float32Array;
		const scores = softmax(logits);

		const tagResults: TagResult[] = [];
		for (let i = 0; i < labels.length; i++) {
			if (scores[i] >= threshold) {
				tagResults.push({
					tag: labelToTag(labels[i]),
					score: Math.round(scores[i] * 1000) / 1000
				});
			}
		}

		tagResults.sort((a, b) => b.score - a.score);
		return tagResults;
	} finally {
		URL.revokeObjectURL(url);
	}
}

export async function tagImageBatch(
	items: { id: string; path: string }[],
	labels: string[] = DEFAULT_LABELS,
	threshold: number = DEFAULT_THRESHOLD
): Promise<Map<string, TagResult[]>> {
	const results = new Map<string, TagResult[]>();

	for (const item of items) {
		try {
			const tagResults = await tagImage(item.path, labels, threshold);
			results.set(item.id, tagResults);
		} catch (error) {
			console.error(`[image-tagger] Failed to tag ${item.path}:`, error);
			results.set(item.id, []);
		}
	}

	return results;
}

export function isTaggerReady(): boolean {
	return ready;
}

export function getTaggerProgress(): TaggerProgress {
	return { ...progress, files: { ...progress.files } };
}

export function initTagger(): void {
	getClassifier();
}

export function getDefaultTags(): string[] {
	return [...DEFAULT_TAGS];
}
