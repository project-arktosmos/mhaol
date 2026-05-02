import {
	firkinsService,
	type Artist,
	type FileEntry,
	type Firkin,
	type FirkinAddon,
	type ImageMeta,
	type Review
} from '$lib/firkins.service';

/// Inputs accepted by `materializeBrowseFirkin`. Mirrors the union of fields
/// the catalog `/visit` resolver and the various related-cards already pass
/// through to `POST /api/firkins`. Optional fields default to empty / null.
export interface BrowseFirkinInput {
	addon: FirkinAddon;
	upstreamId: string | null;
	title: string;
	year?: number | null;
	description?: string | null;
	posterUrl?: string | null;
	backdropUrl?: string | null;
	artistName?: string | null;
	reviews?: Review[];
}

function buildUpstreamFile(addon: FirkinAddon, upstreamId: string): FileEntry | null {
	switch (addon) {
		case 'musicbrainz':
			return {
				type: 'url',
				value: `https://musicbrainz.org/release-group/${upstreamId}`,
				title: 'MusicBrainz Release Group'
			};
		case 'tmdb-tv':
			return {
				type: 'url',
				value: `https://www.themoviedb.org/tv/${upstreamId}`,
				title: 'TMDB TV Show'
			};
		case 'tmdb-movie':
			return {
				type: 'url',
				value: `https://www.themoviedb.org/movie/${upstreamId}`,
				title: 'TMDB Movie'
			};
		case 'youtube-video':
			return {
				type: 'url',
				value: `https://www.youtube.com/watch?v=${upstreamId}`,
				title: 'YouTube Video'
			};
		default:
			return null;
	}
}

/// Create (or look up, via the server's content-address dedup) the
/// non-bookmarked browse-cache firkin for an upstream catalog item. Used by
/// the `/catalog/visit` resolver and by the various "related" cards on the
/// catalog detail page so their `<a href>` can point at `/catalog/<id>`
/// directly instead of bouncing through the visit resolver.
export async function materializeBrowseFirkin(input: BrowseFirkinInput): Promise<Firkin> {
	const title = input.title.trim();
	if (!title) throw new Error('title is required');

	const posterUrl = (input.posterUrl ?? '').trim();
	const backdropUrl = (input.backdropUrl ?? '').trim();
	const description = (input.description ?? '').trim();
	const artistName = (input.artistName ?? '').trim();

	const images: ImageMeta[] = [posterUrl, backdropUrl]
		.filter((u) => u.length > 0)
		.map((u) => ({ url: u, mimeType: 'image/jpeg', fileSize: 0, width: 0, height: 0 }));

	const files: FileEntry[] = [];
	const upstreamId = (input.upstreamId ?? '').trim();
	if (upstreamId) {
		const upstreamFile = buildUpstreamFile(input.addon, upstreamId);
		if (upstreamFile) files.push(upstreamFile);
	}

	const artists: Artist[] = artistName
		? artistName
				.split(/\s*,\s*/)
				.filter((n) => n.length > 0)
				.map((name) => ({ name, role: 'artist' }))
		: [];

	return firkinsService.create({
		title,
		artists,
		description,
		images,
		files,
		year: input.year ?? null,
		addon: input.addon,
		reviews: input.reviews ?? [],
		bookmarked: false
	});
}
