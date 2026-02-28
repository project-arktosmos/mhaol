import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const PUT: RequestHandler = async ({ params, request, locals }) => {
	const item = locals.libraryItemRepo.get(params.itemId);
	if (!item || item.library_id !== params.id) {
		return json({ error: 'Library item not found' }, { status: 404 });
	}

	const body = await request.json();
	const categoryId = body.categoryId;
	if (typeof categoryId !== 'string' || !categoryId.trim()) {
		return json({ error: 'categoryId must be a non-empty string' }, { status: 400 });
	}

	const category = locals.categoryRepo.get(categoryId.trim());
	if (!category) {
		return json({ error: 'Category not found' }, { status: 404 });
	}

	locals.libraryItemRepo.updateCategory(params.itemId, categoryId.trim());

	return json({ ok: true });
};

export const DELETE: RequestHandler = async ({ params, locals }) => {
	const item = locals.libraryItemRepo.get(params.itemId);
	if (!item || item.library_id !== params.id) {
		return json({ error: 'Library item not found' }, { status: 404 });
	}

	locals.libraryItemRepo.clearCategory(params.itemId);

	return json({ ok: true });
};
