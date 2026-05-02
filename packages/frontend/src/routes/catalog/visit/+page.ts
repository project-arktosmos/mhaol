import { error, redirect } from '@sveltejs/kit';
import { base } from '$app/paths';
import {
	firkinsService,
	type FirkinAddon,
	type FileEntry,
	type ImageMeta,
	type Review
} from '$lib/firkins.service';

export const prerender = false;

/// Resolver route: catalog grid clicks land here with the upstream item
/// in the URL query, we POST a non-bookmarked browse-cache firkin against
/// `/api/firkins`, then redirect to the canonical `/catalog/[id]` detail
/// page. The server dedups by content-address so revisits don't mint
/// duplicate records.
export const load = async ({ url }) => {
	const params = url.searchParams;
	const addon = (params.get('addon') ?? '').trim();
	const upstreamId = (params.get('id') ?? '').trim();
	const title = (params.get('title') ?? '').trim();

	if (!addon || !title) {
		throw error(400, 'addon and title are required');
	}

	const yearParam = params.get('year');
	const year =
		yearParam !== null && yearParam !== ''
			? Number.isFinite(Number.parseInt(yearParam, 10))
				? Number.parseInt(yearParam, 10)
				: null
			: null;
	const description = (params.get('description') ?? '').trim();
	const posterUrl = (params.get('posterUrl') ?? '').trim();
	const backdropUrl = (params.get('backdropUrl') ?? '').trim();

	const images: ImageMeta[] = [posterUrl, backdropUrl]
		.filter((u) => u.length > 0)
		.map((u) => ({ url: u, mimeType: 'image/jpeg', fileSize: 0, width: 0, height: 0 }));

	// Mirrors `buildUpstreamSourceFiles()` from the legacy /catalog/virtual
	// page: bake the canonical upstream URL into a `url`-typed file entry
	// so the detail page can extract the upstream id and resume metadata
	// resolution (artists, trailers, tracks).
	const files: FileEntry[] = [];
	if (upstreamId) {
		switch (addon) {
			case 'musicbrainz':
				files.push({
					type: 'url',
					value: `https://musicbrainz.org/release-group/${upstreamId}`,
					title: 'MusicBrainz Release Group'
				});
				break;
			case 'tmdb-tv':
				files.push({
					type: 'url',
					value: `https://www.themoviedb.org/tv/${upstreamId}`,
					title: 'TMDB TV Show'
				});
				break;
			case 'tmdb-movie':
				files.push({
					type: 'url',
					value: `https://www.themoviedb.org/movie/${upstreamId}`,
					title: 'TMDB Movie'
				});
				break;
			case 'youtube-video':
				files.push({
					type: 'url',
					value: `https://www.youtube.com/watch?v=${upstreamId}`,
					title: 'YouTube Video'
				});
				break;
		}
	}

	let reviews: Review[] = [];
	const reviewsParam = params.get('reviews');
	if (reviewsParam) {
		try {
			const parsed = JSON.parse(reviewsParam);
			if (Array.isArray(parsed)) reviews = parsed as Review[];
		} catch {
			// ignore malformed reviews param — the detail page's metadata
			// backfill effect will refetch them via /api/catalog/.../metadata.
		}
	}

	const created = await firkinsService.create({
		title,
		artists: [],
		description,
		images,
		files,
		year,
		addon: addon as FirkinAddon,
		reviews,
		bookmarked: false
	});

	throw redirect(303, `${base}/catalog/${encodeURIComponent(created.id)}`);
};
