export interface CloudDatabaseConfig {
	dbPath?: string;
	walMode?: boolean;
}

export interface CloudSettingRow {
	key: string;
	value: string;
	created_at: string;
	updated_at: string;
}

export interface AttributeTypeRow {
	id: string;
	label: string;
}

export interface CloudLibraryRow {
	id: string;
	name: string;
	path: string;
	kind: string;
	scan_status: 'idle' | 'scanning' | 'error';
	scan_error: string | null;
	item_count: number;
	created_at: string;
	updated_at: string;
}

export interface CloudItemRow {
	id: string;
	library_id: string;
	path: string;
	filename: string;
	extension: string;
	size_bytes: number | null;
	mime_type: string | null;
	checksum: string | null;
	created_at: string;
	updated_at: string;
}

export interface ItemAttributeRow {
	id: string;
	item_id: string;
	key: string;
	value: string;
	attribute_type_id: string;
	source: string;
	confidence: number | null;
	created_at: string;
	updated_at: string;
}

export interface ItemLinkRow {
	id: string;
	item_id: string;
	service: string;
	service_id: string;
	extra: string | null;
	created_at: string;
}

export interface CloudCollectionRow {
	id: string;
	library_id: string;
	name: string;
	description: string | null;
	cover_path: string | null;
	kind: 'manual' | 'auto' | 'smart';
	created_at: string;
	updated_at: string;
}

export interface CollectionItemRow {
	id: string;
	collection_id: string;
	item_id: string;
	position: number;
	created_at: string;
}

export interface SignalingServerRow {
	id: string;
	name: string;
	url: string;
	enabled: number;
	created_at: string;
	updated_at: string;
}
