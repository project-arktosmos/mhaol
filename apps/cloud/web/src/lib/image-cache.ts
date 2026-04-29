export function cachedImageUrl(url: string | null | undefined): string {
	if (!url) return '';
	if (!/^https?:\/\//i.test(url)) return url;
	return `/api/image-cache?url=${encodeURIComponent(url)}`;
}
