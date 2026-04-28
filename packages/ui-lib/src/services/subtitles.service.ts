import { writable, type Writable } from 'svelte/store';
import { fetchJson, resolveApiUrl } from 'ui-lib/transport/fetch-helpers';
import type {
	AssignedSubtitle,
	SubtitleSearchContext,
	SubtitleSearchResult
} from 'ui-lib/types/subtitles.type';
import { buildMediaKey } from 'ui-lib/types/subtitles.type';

interface SubtitlesState {
	context: SubtitleSearchContext | null;
	searching: boolean;
	results: SubtitleSearchResult[];
	assigned: AssignedSubtitle[];
	downloading: string | null;
	error: string | null;
}

const initialState: SubtitlesState = {
	context: null,
	searching: false,
	results: [],
	assigned: [],
	downloading: null,
	error: null
};

class SubtitlesService {
	public state: Writable<SubtitlesState> = writable(initialState);

	setContext(ctx: SubtitleSearchContext | null): void {
		this.state.update((s) => ({ ...s, context: ctx, results: [], error: null }));
		if (ctx) {
			this.refreshAssigned(ctx).catch(() => {});
		} else {
			this.state.update((s) => ({ ...s, assigned: [] }));
		}
	}

	async refreshAssigned(ctx: SubtitleSearchContext): Promise<AssignedSubtitle[]> {
		const mediaKey = buildMediaKey(ctx);
		try {
			const list = await fetchJson<AssignedSubtitle[]>(
				`/api/subtitles?mediaKey=${encodeURIComponent(mediaKey)}`
			);
			const assigned = list.map((row) => ({ ...row, url: this.toAbsolute(row.url) }));
			this.state.update((s) => ({ ...s, assigned }));
			return assigned;
		} catch (e) {
			const msg = e instanceof Error ? e.message : String(e);
			this.state.update((s) => ({ ...s, error: msg }));
			return [];
		}
	}

	async search(languages?: string[], hearingImpaired?: boolean): Promise<void> {
		let ctx: SubtitleSearchContext | null = null;
		this.state.update((s) => {
			ctx = s.context;
			return { ...s, searching: true, error: null };
		});
		if (!ctx) {
			this.state.update((s) => ({ ...s, searching: false, error: 'No search context' }));
			return;
		}
		try {
			const results = await fetchJson<SubtitleSearchResult[]>('/api/subtitles/search', {
				method: 'POST',
				body: JSON.stringify({
					type: (ctx as SubtitleSearchContext).type,
					tmdbId: (ctx as SubtitleSearchContext).tmdbId,
					imdbId: (ctx as SubtitleSearchContext).imdbId,
					season: (ctx as SubtitleSearchContext).season,
					episode: (ctx as SubtitleSearchContext).episode,
					languages,
					hearingImpaired
				})
			});
			this.state.update((s) => ({ ...s, searching: false, results }));
		} catch (e) {
			const msg = e instanceof Error ? e.message : String(e);
			this.state.update((s) => ({ ...s, searching: false, error: msg }));
		}
	}

	async download(result: SubtitleSearchResult): Promise<AssignedSubtitle | null> {
		let ctx: SubtitleSearchContext | null = null;
		this.state.update((s) => {
			ctx = s.context;
			return { ...s, downloading: result.id, error: null };
		});
		if (!ctx) {
			this.state.update((s) => ({ ...s, downloading: null, error: 'No search context' }));
			return null;
		}
		try {
			const row = await fetchJson<AssignedSubtitle>('/api/subtitles/download', {
				method: 'POST',
				body: JSON.stringify({
					mediaKey: buildMediaKey(ctx as SubtitleSearchContext),
					url: result.url,
					languageCode: result.language,
					languageName: result.display || result.language,
					source: `wyzie:${result.source || 'unknown'}`,
					sourceId: result.id,
					format: result.format,
					hearingImpaired: result.isHearingImpaired
				})
			});
			const enriched: AssignedSubtitle = { ...row, url: this.toAbsolute(row.url) };
			this.state.update((s) => ({
				...s,
				downloading: null,
				assigned: [enriched, ...s.assigned]
			}));
			return enriched;
		} catch (e) {
			const msg = e instanceof Error ? e.message : String(e);
			this.state.update((s) => ({ ...s, downloading: null, error: msg }));
			return null;
		}
	}

	async remove(id: string): Promise<void> {
		try {
			await fetchJson<{ ok: boolean }>(`/api/subtitles/${encodeURIComponent(id)}`, {
				method: 'DELETE'
			});
			this.state.update((s) => ({
				...s,
				assigned: s.assigned.filter((a) => a.id !== id)
			}));
		} catch (e) {
			const msg = e instanceof Error ? e.message : String(e);
			this.state.update((s) => ({ ...s, error: msg }));
		}
	}

	private toAbsolute(url: string): string {
		if (/^https?:/.test(url)) return url;
		return resolveApiUrl(url);
	}
}

export const subtitlesService = new SubtitlesService();
