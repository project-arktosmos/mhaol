import type { MediaItem } from 'ui-lib/types/media-card.type';
import type { DisplayTMDBMovieDetails } from 'addons/tmdb/types';

// ===== Server Catalog Protocol =====

export interface CatalogMovie {
	item: MediaItem;
	tmdb: DisplayTMDBMovieDetails | null;
	streamable: boolean;
}

export interface ServerCatalogStartMessage {
	type: 'catalog-start';
	count: number;
}

export interface ServerCatalogMoviesMessage {
	type: 'catalog-movies';
	movies: CatalogMovie[];
}

export interface ServerCatalogStreamRequestMessage {
	type: 'stream-request';
	tmdbId: number;
}

export interface ServerCatalogStreamSessionMessage {
	type: 'stream-session';
	sessionId: string;
	roomId: string;
	signalingUrl: string;
}

export interface ServerCatalogStreamErrorMessage {
	type: 'stream-error';
	error: string;
}

export interface ServerCatalogRequestMessage {
	type: 'catalog-request';
}

export type ServerCatalogMessage =
	| ServerCatalogStartMessage
	| ServerCatalogMoviesMessage
	| ServerCatalogStreamRequestMessage
	| ServerCatalogStreamSessionMessage
	| ServerCatalogStreamErrorMessage
	| ServerCatalogRequestMessage;

export interface DataChannelServerCatalogEnvelope {
	channel: 'server-catalog';
	payload: ServerCatalogMessage;
}

// ===== Service State =====

export interface ServerCatalogState {
	movies: Record<string, CatalogMovie[]>;
}
