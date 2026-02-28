import type { Database as DatabaseType } from 'better-sqlite3';
import type {
	SettingsRepository,
	MetadataRepository,
	LibraryRepository
} from 'database/repositories';

// --- JSON manifest types (matches plugin.json structure) ---

export interface PluginCompatibility {
	mobile: boolean;
	computer: boolean;
}

export interface PluginManifest {
	name: string;
	version: string;
	description: string;

	source?: 'plugin' | 'addon';

	compatibility?: PluginCompatibility;

	processes?: PluginProcessManifest[];

	schema?: {
		sql: string;
	};

	settings?: PluginSettingManifest[];

	locals?: PluginLocalManifest[];
}

export interface PluginProcessManifest {
	id: string;
	binary: string;
	binaryEnv?: string;
	args?: string[];
	cwd?: string;
	port: number;
	portEnv?: string;
	env?: Record<string, string | PluginEnvResolver>;
	logPrefix: string;
	healthCheck?: {
		path: string;
		retries?: number;
		intervalMs?: number;
	};
}

export type PluginEnvResolver = PluginEnvLibraryPath | PluginEnvSettingValue;

export interface PluginEnvLibraryPath {
	type: 'libraryPath';
	metaKey: string;
	fallback: string;
}

export interface PluginEnvSettingValue {
	type: 'settingValue';
	key: string;
	optional?: boolean;
}

export interface PluginSettingManifest {
	key: string;
	default: string;
	envKey?: string;
}

export interface PluginLocalManifest {
	key: string;
	source: 'processUrl' | 'processAvailable';
	processId: string;
}

// --- TypeScript companion types (from plugin.ts) ---

export interface PluginLinkSource {
	service: string;
	label: string;
	mediaTypeId: string;
	categoryId?: string | null;
}

export interface PluginCompanion {
	repositories?: PluginRepository[];

	locals?: Record<string, (ctx: PluginContext) => unknown>;

	linkSources?: PluginLinkSource[];

	onInit?: (ctx: PluginContext) => Promise<void>;
	onShutdown?: (ctx: PluginContext) => void;

	scheduledTasks?: PluginScheduledTask[];

	schema?: {
		migrations?: (db: DatabaseType) => void;
	};
}

export interface PluginRepository {
	class: new (db: DatabaseType) => unknown;
	localsKey: string;
}

export interface PluginScheduledTask {
	id: string;
	intervalMs: number;
	handler: (ctx: PluginContext) => Promise<void>;
}

// --- Context provided to plugin hooks ---

export interface PluginContext {
	db: DatabaseType;
	settingsRepo: SettingsRepository;
	metadataRepo: MetadataRepository;
	libraryRepo: LibraryRepository;
	getProcessUrl: (processId: string) => string;
	isProcessAvailable: (processId: string) => boolean;
	getRepository: <T = unknown>(localsKey: string) => T;
}

// --- Status types (returned by getStatus()) ---

export interface PluginProcessStatus {
	id: string;
	available: boolean;
	port: number;
	url: string;
	logPrefix: string;
}

export interface PluginSettingStatus {
	key: string;
	value: string;
	default: string;
}

export interface PluginStatus {
	name: string;
	version: string;
	description: string;
	source: 'plugin' | 'addon';
	compatibility: PluginCompatibility;
	processes: PluginProcessStatus[];
	settings: PluginSettingStatus[];
	scheduledTasks: string[];
	schemaTables: { name: string; columns: string[] }[];
	linkSources: PluginLinkSource[];
}

// --- Internal: combined registration ---

export interface PluginRegistration {
	manifest: PluginManifest;
	companion?: PluginCompanion;
}
