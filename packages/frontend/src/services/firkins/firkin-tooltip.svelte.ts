import type { FirkinReview } from 'cloud-ui';

export interface FirkinTooltipContent {
	title: string;
	description?: string | null;
	imageUrl?: string | null;
	reviews?: FirkinReview[];
}

class FirkinTooltipService {
	content = $state<FirkinTooltipContent | null>(null);
	pointer = $state<{ x: number; y: number }>({ x: 0, y: 0 });

	show(content: FirkinTooltipContent, x: number, y: number) {
		this.content = content;
		this.pointer = { x, y };
	}

	move(x: number, y: number) {
		this.pointer = { x, y };
	}

	hide() {
		this.content = null;
	}
}

export const firkinTooltipService = new FirkinTooltipService();
