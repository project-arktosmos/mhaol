export interface Pin {
	id: string;
	service: string;
	serviceId: string;
	label: string;
	createdAt: string;
}

export interface PinsState {
	loading: boolean;
	items: Pin[];
	error: string | null;
}
