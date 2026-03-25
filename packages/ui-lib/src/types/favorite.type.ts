export interface Favorite {
	id: string;
	wallet: string;
	service: string;
	serviceId: string;
	label: string;
	createdAt: string;
}

export interface FavoritesState {
	loading: boolean;
	items: Favorite[];
	error: string | null;
}
