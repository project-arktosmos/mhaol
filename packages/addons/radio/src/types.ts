// Raw Radio Browser API types (https://api.radio-browser.info/)

export interface RadioBrowserStation {
	changeuuid: string;
	stationuuid: string;
	name: string;
	url: string;
	url_resolved: string;
	homepage: string;
	favicon: string;
	tags: string;
	country: string;
	countrycode: string;
	state?: string;
	language: string;
	languagecodes?: string;
	votes: number;
	lastchangetime?: string;
	codec: string;
	bitrate: number;
	hls: number;
	lastcheckok: number;
	lastchecktime?: string;
	clickcount: number;
	clicktrend: number;
	geo_lat?: number | null;
	geo_long?: number | null;
}

export interface RadioBrowserTag {
	name: string;
	stationcount: number;
}

export interface RadioBrowserCountry {
	name: string;
	iso_3166_1?: string;
	stationcount: number;
}

export interface RadioBrowserLanguage {
	name: string;
	iso_639?: string;
	stationcount: number;
}

// Display types (camelCase, transformed for UI)

export interface DisplayRadioStation {
	id: string;
	name: string;
	streamUrl: string;
	homepage: string | null;
	logo: string | null;
	tags: string[];
	country: string;
	countryCode: string;
	language: string;
	codec: string | null;
	bitrate: number | null;
	isHls: boolean;
	votes: number;
	clickCount: number;
}

export interface DisplayRadioSearchResult {
	stations: DisplayRadioStation[];
	page: number;
	limit: number;
}

export interface RadioSearchOptions {
	tag?: string;
	country?: string;
	countryCode?: string;
	language?: string;
	page?: number;
	limit?: number;
	hideBroken?: boolean;
}
