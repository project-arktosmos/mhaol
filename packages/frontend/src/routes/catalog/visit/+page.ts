import { error, redirect } from '@sveltejs/kit';
import { base } from '$app/paths';
import { materializeBrowseFirkin } from '$lib/catalog-firkin';
import type { FirkinAddon, Review } from '$lib/firkins.service';

export const prerender = false;

/// Resolver route: catalog grid clicks land here with the upstream item
/// in the URL query, we POST a non-bookmarked browse-cache firkin against
/// `/api/firkins`, then redirect to the canonical `/catalog/[id]` detail
/// page. The server dedups by content-address so revisits don't mint
/// duplicate records. Related-card clicks bypass this route entirely —
/// see `materializeBrowseFirkin` in `$lib/catalog-firkin`, which the
/// related cards call eagerly so their hrefs already point at /catalog/<id>.
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

	const created = await materializeBrowseFirkin({
		addon: addon as FirkinAddon,
		upstreamId,
		title,
		year,
		description: params.get('description') ?? '',
		posterUrl: params.get('posterUrl') ?? '',
		backdropUrl: params.get('backdropUrl') ?? '',
		artistName: params.get('artistName') ?? '',
		reviews
	});

	throw redirect(303, `${base}/catalog/${encodeURIComponent(created.id)}`);
};
