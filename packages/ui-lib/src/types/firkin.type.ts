/**
 * Artist record with two faces:
 *
 * - **Inbound** (catalog/search responses, firkin POST/PUT bodies): a
 *   single-occurrence shape carrying one optional `role`. The cloud
 *   merges these into the canonical `artist` table by name, so multiple
 *   inbound calls for the same person accumulate into one record.
 * - **Resolved** (firkin GET responses, `/api/artists` rows): the
 *   canonical multi-role shape. `roles` is the deduped union of every
 *   role/title the artist has been recorded with.
 *
 * Both faces share the same TypeScript type so callers don't have to
 * juggle two — fields are populated as needed at each end.
 */
export interface FirkinArtist {
	/** CID of the persisted `artist` doc. Absent on transient catalog/search responses. */
	id?: string;
	name: string;
	/** Single-occurrence role used on the inbound side (catalog/search → firkin upsert). */
	role?: string;
	/** Canonical multi-role array on the resolved side. Deduped server-side. */
	roles?: string[];
	imageUrl?: string;
}

export interface FirkinImage {
	url: string;
	mimeType: string;
	fileSize: number;
	width: number;
	height: number;
}

export type FirkinFileType = 'ipfs' | 'torrent magnet' | 'url';

export interface FirkinFile {
	type: FirkinFileType;
	value: string;
	title?: string;
}

export interface CloudFirkin {
	id: string;
	title: string;
	/** CIDs of the referenced artist docs, in order. Persisted on the firkin body — drives its own CID. */
	artistIds?: string[];
	/** Resolved artist bodies (server-side join from `artistIds`). */
	artists: FirkinArtist[];
	description: string;
	images: FirkinImage[];
	files: FirkinFile[];
	year: number | null;
	/**
	 * The addon id this firkin was sourced from (e.g. `tmdb-movie`,
	 * `tmdb-tv`, `musicbrainz`, `local-album`). Each addon is bound to a
	 * single firkin kind, so the addon implies the kind — there is no
	 * separate `type` field.
	 */
	addon: string;
	/**
	 * EVM address of the account that created the firkin. Filled from the
	 * browser-resident user identity (see `userIdentityService`) on
	 * user-initiated creates; empty string for server-side auto-creates
	 * (library scan).
	 */
	creator: string;
	created_at: string;
	updated_at: string;
	/** Rolling-forward nonce. Older records without this field are treated as 0. */
	version?: number;
	/** CIDs of every prior version, oldest first. `version_hashes.length === version` is the chain integrity invariant. */
	version_hashes?: string[];
}
