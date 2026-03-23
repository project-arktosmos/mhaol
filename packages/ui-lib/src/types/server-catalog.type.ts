import type { MediaItem } from 'ui-lib/types/media-card.type';
import type { DisplayTMDBMovieDetails } from 'addons/tmdb/types';

// ===== Server Catalog Protocol =====

export interface CatalogMovie {
	item: MediaItem;
	tmdb: DisplayTMDBMovieDetails | null;
}

export interface ServerCatalogMoviesMessage {
	type: 'catalog-movies';
	movies: CatalogMovie[];
}

export interface ServerCatalogStreamRequestMessage {
	type: 'stream-request';
	itemPath: string;
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

export type ServerCatalogMessage =
	| ServerCatalogMoviesMessage
	| ServerCatalogStreamRequestMessage
	| ServerCatalogStreamSessionMessage
	| ServerCatalogStreamErrorMessage;

export interface DataChannelServerCatalogEnvelope {
	channel: 'server-catalog';
	payload: ServerCatalogMessage;
}

// ===== Service State =====

export interface ServerCatalogState {
	movies: Record<string, CatalogMovie[]>;
}
