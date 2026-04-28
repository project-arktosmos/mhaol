import { writable, get, type Writable } from 'svelte/store';
import { locale } from 'svelte-i18n';
import { fetchJson, resolveApiUrl } from 'ui-lib/transport/fetch-helpers';
import type {
	AssignedSubtitle,
	SubtitleSearchContext,
	SubtitleSearchResult
} from 'ui-lib/types/subtitles.type';
import { buildMediaKey } from 'ui-lib/types/subtitles.type';

/// Default search languages for a context: title's original language (from TMDB)
/// plus the user's current UI locale, deduped, lowercased, 2-letter. Empty array
/// means "no preference" (search will return all available languages).
function defaultLanguagesFor(ctx: SubtitleSearchContext): string[] {
	const out = new Set<string>();
	const orig = ctx.originalLanguage?.trim().toLowerCase().slice(0, 2);
	if (orig) out.add(orig);
	const ui = get(locale);
	const uiShort = typeof ui === 'string' ? ui.trim().toLowerCase().slice(0, 2) : '';
	if (uiShort) out.add(uiShort);
	return Array.from(out);
}

interface SubtitlesState {
	context: SubtitleSearchContext | null;
	searching: boolean;
	results: SubtitleSearchResult[];
	assigned: AssignedSubtitle[];
	downloading: string | null;
	error: string | null;
	/// Languages used for the most recent search.
	lastLanguages: string[];
}

const initialState: SubtitlesState = {
	context: null,
	searching: false,
	results: [],
	assigned: [],
	downloading: null,
	error: null,
	lastLanguages: []
};

class SubtitlesService {
	public state: Writable<SubtitlesState> = writable(initialState);

	setContext(ctx: SubtitleSearchContext | null): void {
		const prev = get(this.state).context;
		const sameContext =
			prev !== null &&
			ctx !== null &&
			prev.type === ctx.type &&
			prev.tmdbId === ctx.tmdbId &&
			prev.season === ctx.season &&
			prev.episode === ctx.episode;
		if (sameContext) return;
		this.state.update((s) => ({ ...s, context: ctx, results: [], error: null }));
		if (ctx) {
			this.refreshAssigned(ctx).catch(() => {});
			// Kick off an initial search using the title's original language and the user's UI locale.
			const langs = defaultLanguagesFor(ctx);
			this.search(langs.length ? langs : undefined).catch(() => {});
		} else {
			this.state.update((s) => ({ ...s, assigned: [], lastLanguages: [] }));
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
			this.state.update((s) => ({
				...s,
				searching: false,
				results,
				lastLanguages: languages ?? []
			}));
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

	defaultLanguages(ctx: SubtitleSearchContext): string[] {
		return defaultLanguagesFor(ctx);
	}

	private toAbsolute(url: string): string {
		if (/^https?:/.test(url)) return url;
		return resolveApiUrl(url);
	}
}

export const subtitlesService = new SubtitlesService();
