export interface MediaBarContext {
	configure(config: { title: string; count?: number | null; countLabel?: string }): void;
	readonly controlsTarget: HTMLDivElement | undefined;
	readonly tabsTarget: HTMLDivElement | undefined;
	readonly filterBarTarget: HTMLDivElement | undefined;
}

export const MEDIA_BAR_KEY = 'mediaBar';
