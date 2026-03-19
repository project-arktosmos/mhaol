import { apiUrl } from '$lib/api-base';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch }) => {
	const res = await fetch(apiUrl('/api/media'));
	return res.json();
};
