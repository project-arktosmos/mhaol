// Raw iptv-org API types

export interface IptvOrgChannel {
	id: string;
	name: string;
	alt_names?: string[];
	network?: string | null;
	owners?: string[];
	country: string;
	subdivision?: string | null;
	city?: string | null;
	categories?: string[];
	is_nsfw?: boolean;
	launched?: string | null;
	closed?: string | null;
	replaced_by?: string | null;
	website?: string | null;
}

export interface IptvOrgStream {
	channel: string | null;
	feed?: string | null;
	url: string;
	referrer?: string | null;
	user_agent?: string | null;
	quality?: string | null;
}

export interface IptvOrgLogo {
	channel: string;
	feed?: string | null;
	tags?: string[];
	width?: number;
	height?: number;
	format?: string;
	url: string;
}

export interface IptvOrgCategory {
	id: string;
	name: string;
}

export interface IptvOrgCountry {
	code: string;
	name: string;
	flag?: string;
	languages?: string[];
}

export interface IptvOrgLanguage {
	code: string;
	name: string;
}

export interface IptvOrgGuide {
	channel: string | null;
	feed?: string | null;
	site: string;
	site_id: string;
	site_name?: string;
	lang?: string;
}

// Display types — what the frontend operates on

export interface DisplayIptvChannel {
	id: string;
	name: string;
	country: string;
	categories: string[];
	logo: string | null;
	website: string | null;
	isNsfw: boolean;
	hasEpg: boolean;
}

export interface DisplayIptvStream {
	channel: string;
	url: string;
	httpReferrer: string | null;
	userAgent: string | null;
	quality: string | null;
}

export interface DisplayIptvSearchResult {
	channels: DisplayIptvChannel[];
	total: number;
	page: number;
	limit: number;
}

export interface IptvSearchOptions {
	category?: string;
	country?: string;
	hasEpg?: boolean;
	page?: number;
	limit?: number;
}
