import { writable, get, type Writable } from 'svelte/store';
import { browser } from '$app/environment';

export const REACTION_EMOJIS = ['👎', '⭐', '👍'] as const;
export type ReactionEmoji = (typeof REACTION_EMOJIS)[number];

const STORAGE_KEY = 'mhaol:firkin-reactions';

function load(): Record<string, ReactionEmoji> {
	if (!browser) return {};
	try {
		const raw = localStorage.getItem(STORAGE_KEY);
		if (!raw) return {};
		const parsed = JSON.parse(raw) as unknown;
		if (!parsed || typeof parsed !== 'object') return {};
		const result: Record<string, ReactionEmoji> = {};
		for (const [k, v] of Object.entries(parsed as Record<string, unknown>)) {
			if (typeof v === 'string' && (REACTION_EMOJIS as readonly string[]).includes(v)) {
				result[k] = v as ReactionEmoji;
			}
		}
		return result;
	} catch {
		return {};
	}
}

function persist(state: Record<string, ReactionEmoji>): void {
	if (!browser) return;
	try {
		localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
	} catch {
		// ignore quota / serialization errors
	}
}

class FirkinReactionsService {
	state: Writable<Record<string, ReactionEmoji>> = writable(load());

	set(firkinId: string, emoji: ReactionEmoji): void {
		this.state.update((s) => {
			const next = { ...s, [firkinId]: emoji };
			persist(next);
			return next;
		});
	}

	clear(firkinId: string): void {
		this.state.update((s) => {
			if (!(firkinId in s)) return s;
			const next = { ...s };
			delete next[firkinId];
			persist(next);
			return next;
		});
	}

	get(firkinId: string): ReactionEmoji | null {
		return get(this.state)[firkinId] ?? null;
	}
}

export const firkinReactionsService = new FirkinReactionsService();
