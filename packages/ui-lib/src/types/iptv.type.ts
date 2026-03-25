export interface IptvChannel {
	id: string;
	name: string;
	country: string;
	categories: string[];
	logo: string | null;
	website: string | null;
	isNsfw: boolean;
}

export interface IptvStream {
	channel: string;
	url: string;
	httpReferrer: string | null;
	userAgent: string | null;
}

export interface IptvCategory {
	id: string;
	name: string;
}

export interface IptvCountry {
	code: string;
	name: string;
}

export interface IptvChannelDetail {
	channel: IptvChannel;
	streams: IptvStream[];
}

export interface IptvSearchResult {
	channels: IptvChannel[];
	total: number;
	page: number;
	limit: number;
}

export interface IptvServiceState {
	initialized: boolean;
	loading: boolean;
	error: string | null;
	channels: IptvChannel[];
	total: number;
	page: number;
	categories: IptvCategory[];
	countries: IptvCountry[];
	query: string;
	selectedCategory: string;
	selectedCountry: string;
}
