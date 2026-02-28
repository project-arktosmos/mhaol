import type { Database as DatabaseType } from 'better-sqlite3';
import type {
	SettingsRepository,
	MetadataRepository,
	LibraryRepository
} from 'database/repositories';
import { LinkSourceRepository } from 'database/repositories';
import { spawn, type ChildProcess } from 'node:child_process';
import { existsSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import type {
	PluginManifest,
	PluginCompanion,
	PluginRegistration,
	PluginContext,
	PluginProcessManifest,
	PluginEnvResolver,
	PluginStatus
} from './types';

const __dirname = dirname(fileURLToPath(import.meta.url));
const PACKAGE_ROOT = join(__dirname, '..', '..', '..', '..');

interface CoreRepos {
	settingsRepo: SettingsRepository;
	metadataRepo: MetadataRepository;
	libraryRepo: LibraryRepository;
}

interface ProcessState {
	process: ChildProcess | null;
	available: boolean;
	port: number;
	url: string;
}

export class PluginConnector {
	private db: DatabaseType;
	private coreRepos: CoreRepos;
	private linkSourceRepo: LinkSourceRepository;
	private registrations: PluginRegistration[] = [];
	private processes: Map<string, ProcessState> = new Map();
	private repositories: Map<string, unknown> = new Map();
	private scheduledTasks: Map<string, NodeJS.Timeout> = new Map();
	private initialized = false;

	constructor(db: DatabaseType, coreRepos: CoreRepos) {
		this.db = db;
		this.coreRepos = coreRepos;
		this.linkSourceRepo = new LinkSourceRepository(db);
	}

	register(manifest: PluginManifest, companion?: PluginCompanion): void {
		if (this.initialized) {
			throw new Error(`Cannot register plugin "${manifest.name}" after initialization`);
		}
		this.registrations.push({ manifest, companion });
	}

	async initialize(): Promise<void> {
		if (this.initialized) {
			throw new Error('PluginConnector already initialized');
		}

		for (const reg of this.registrations) {
			const { manifest, companion } = reg;
			const pluginLog = `[plugin:${manifest.name}]`;

			// 1. Run schema SQL
			if (manifest.schema?.sql) {
				this.db.exec(manifest.schema.sql);
				console.log(`${pluginLog} Schema applied`);
			}

			// 2. Run migrations
			if (companion?.schema?.migrations) {
				companion.schema.migrations(this.db);
			}

			// 3. Seed settings defaults
			if (manifest.settings && manifest.settings.length > 0) {
				this.seedSettings(manifest.settings);
			}

			// 4. Instantiate repositories
			if (companion?.repositories) {
				for (const repoDef of companion.repositories) {
					const instance = new repoDef.class(this.db);
					this.repositories.set(repoDef.localsKey, instance);
				}
			}

			// 5. Register link sources
			if (companion?.linkSources) {
				for (const ls of companion.linkSources) {
					this.linkSourceRepo.upsert({
						id: crypto.randomUUID(),
						plugin: manifest.name,
						service: ls.service,
						label: ls.label,
						media_type_id: ls.mediaTypeId,
						category_id: ls.categoryId ?? null
					});
				}
			}

			// 6. Spawn declared processes
			if (manifest.processes) {
				for (const proc of manifest.processes) {
					this.spawnPluginProcess(proc, pluginLog);
				}
			}

			// 7. Run onInit hook
			if (companion?.onInit) {
				const ctx = this.buildContext();
				try {
					await companion.onInit(ctx);
				} catch (e) {
					console.error(`${pluginLog} onInit failed: ${e}`);
				}
			}

			// 8. Start scheduled tasks
			if (companion?.scheduledTasks) {
				const ctx = this.buildContext();
				for (const task of companion.scheduledTasks) {
					const interval = setInterval(async () => {
						try {
							await task.handler(ctx);
						} catch (e) {
							console.warn(`${pluginLog} Task "${task.id}" failed: ${e}`);
						}
					}, task.intervalMs);
					this.scheduledTasks.set(`${manifest.name}:${task.id}`, interval);
				}
			}

			console.log(`${pluginLog} Initialized (v${manifest.version})`);
		}

		this.initialized = true;
	}

	getLinkSourceRepo(): LinkSourceRepository {
		return this.linkSourceRepo;
	}

	getLocals(): Record<string, unknown> {
		const locals: Record<string, unknown> = {};

		// Repository instances
		for (const [key, instance] of this.repositories) {
			locals[key] = instance;
		}

		// Manifest-declared locals (static process lookups)
		for (const reg of this.registrations) {
			if (reg.manifest.locals) {
				for (const local of reg.manifest.locals) {
					const state = this.processes.get(local.processId);
					if (local.source === 'processUrl') {
						locals[local.key] = state?.url ?? '';
					} else if (local.source === 'processAvailable') {
						locals[local.key] = state?.available ?? false;
					}
				}
			}

			// Companion-declared locals (dynamic providers)
			if (reg.companion?.locals) {
				const ctx = this.buildContext();
				for (const [key, provider] of Object.entries(reg.companion.locals)) {
					locals[key] = provider(ctx);
				}
			}
		}

		return locals;
	}

	getStatus(): PluginStatus[] {
		return this.registrations.map((reg) => {
			const { manifest, companion } = reg;

			const processes = (manifest.processes ?? []).map((proc) => {
				const state = this.processes.get(proc.id);
				return {
					id: proc.id,
					available: state?.available ?? false,
					port: state?.port ?? proc.port,
					url: state?.url ?? '',
					logPrefix: proc.logPrefix
				};
			});

			const settings = (manifest.settings ?? []).map((s) => ({
				key: s.key,
				value: this.coreRepos.settingsRepo.get(s.key) ?? s.default,
				default: s.default
			}));

			const scheduledTasks = (companion?.scheduledTasks ?? []).map((t) => t.id);

			const linkSources = companion?.linkSources ?? [];

			const schemaTables = manifest.schema?.sql
				? [...manifest.schema.sql.matchAll(/CREATE\s+TABLE\s+(?:IF\s+NOT\s+EXISTS\s+)?(\w+)\s*\(([^)]+)\)/gi)].map(m => ({
					name: m[1],
					columns: m[2]
						.split(',')
						.map(c => c.trim().split(/\s+/)[0])
						.filter(c => c && !c.toUpperCase().startsWith('PRIMARY') && !c.toUpperCase().startsWith('FOREIGN') && !c.toUpperCase().startsWith('UNIQUE') && !c.toUpperCase().startsWith('CHECK') && !c.toUpperCase().startsWith('CONSTRAINT'))
				}))
				: [];

			return {
				name: manifest.name,
				version: manifest.version,
				description: manifest.description,
				source: manifest.source ?? 'plugin',
				compatibility: manifest.compatibility ?? { mobile: false, computer: true },
				processes,
				settings,
				scheduledTasks,
				schemaTables,
				linkSources
			};
		});
	}

	updatePluginSetting(pluginName: string, key: string, value: string): boolean {
		const reg = this.registrations.find((r) => r.manifest.name === pluginName);
		if (!reg) return false;

		const validKeys = (reg.manifest.settings ?? []).map((s) => s.key);
		if (!validKeys.includes(key)) return false;

		this.coreRepos.settingsRepo.set(key, value);
		return true;
	}

	shutdown(): void {
		// Reverse order shutdown
		for (const reg of [...this.registrations].reverse()) {
			const pluginLog = `[plugin:${reg.manifest.name}]`;

			// Call onShutdown
			if (reg.companion?.onShutdown) {
				try {
					const ctx = this.buildContext();
					reg.companion.onShutdown(ctx);
				} catch (e) {
					console.error(`${pluginLog} onShutdown failed: ${e}`);
				}
			}
		}

		// Clear all scheduled tasks
		for (const [id, interval] of this.scheduledTasks) {
			clearInterval(interval);
		}
		this.scheduledTasks.clear();

		// Kill all spawned processes
		for (const [id, state] of this.processes) {
			if (state.process) {
				state.process.kill();
				state.process = null;
				state.available = false;
			}
		}
	}

	private spawnPluginProcess(proc: PluginProcessManifest, pluginLog: string): void {
		const isBareCommand = !proc.binary.includes('/') && !proc.binary.includes('\\');
		const binaryPath = isBareCommand
			? proc.binary
			: proc.binaryEnv
				? process.env[proc.binaryEnv] ?? join(PACKAGE_ROOT, proc.binary)
				: join(PACKAGE_ROOT, proc.binary);

		const port = proc.portEnv ? process.env[proc.portEnv] ?? String(proc.port) : String(proc.port);

		const url = `http://localhost:${port}`;
		const state: ProcessState = { process: null, available: false, port: Number(port), url };
		this.processes.set(proc.id, state);

		if (!isBareCommand && !existsSync(binaryPath)) {
			console.warn(
				`${pluginLog} Binary not found at ${binaryPath}, ${proc.logPrefix} disabled`
			);
			return;
		}

		const resolvedEnv: Record<string, string> = {
			...process.env
		};

		// Resolve env values (static strings act as defaults; process.env takes precedence)
		if (proc.env) {
			for (const [key, value] of Object.entries(proc.env)) {
				if (typeof value === 'string' && process.env[key]) {
					// Static default — process.env takes precedence
					continue;
				}
				const resolved = this.resolveEnvValue(value);
				if (resolved !== null) {
					resolvedEnv[key] = resolved;
				}
			}
		}

		// Inject port env var matching the portEnv key
		if (proc.portEnv) {
			resolvedEnv[proc.portEnv] = port;
		}

		const child = spawn(binaryPath, proc.args ?? [], {
			cwd: PACKAGE_ROOT,
			env: resolvedEnv,
			stdio: ['ignore', 'pipe', 'pipe']
		});

		state.process = child;
		state.available = true;

		child.stdout?.on('data', (data: Buffer) => {
			for (const line of data.toString().trimEnd().split('\n')) {
				console.log(`[${proc.logPrefix}] ${line}`);
			}
		});

		child.stderr?.on('data', (data: Buffer) => {
			for (const line of data.toString().trimEnd().split('\n')) {
				console.error(`[${proc.logPrefix}] ${line}`);
			}
		});

		child.on('error', (err) => {
			console.error(`[${proc.logPrefix}] Failed to start: ${err.message}`);
			state.available = false;
		});

		child.on('exit', (code) => {
			console.log(`[${proc.logPrefix}] Process exited with code ${code}`);
			state.available = false;
			state.process = null;
		});

		console.log(
			`[${proc.logPrefix}] Started on port ${port} (pid: ${child.pid})`
		);
	}

	private resolveEnvValue(value: string | PluginEnvResolver): string | null {
		if (typeof value === 'string') {
			return value;
		}

		if (value.type === 'libraryPath') {
			return this.resolveLibraryPath(value.metaKey, value.fallback);
		}

		if (value.type === 'settingValue') {
			const settingVal = this.coreRepos.settingsRepo.get(value.key);
			if (!settingVal && value.optional) return null;
			return settingVal ?? '';
		}

		return '';
	}

	private resolveLibraryPath(metaKey: string, fallback: string): string {
		const libId = this.coreRepos.metadataRepo.getValue<string>(metaKey) as string | undefined;
		if (libId) {
			const lib = this.coreRepos.libraryRepo.get(libId);
			if (lib) return lib.path;
		}
		// Fallback: use first library if available
		const allLibs = this.coreRepos.libraryRepo.getAll();
		if (allLibs.length > 0) return allLibs[0].path;

		// Resolve ~ to HOME
		if (fallback.startsWith('~/')) {
			return join(process.env.HOME ?? '/tmp', fallback.slice(2));
		}
		return fallback;
	}

	private seedSettings(settings: Array<{ key: string; default: string; envKey?: string }>): void {
		const entries: Record<string, string> = {};
		for (const setting of settings) {
			const envValue = setting.envKey ? process.env[setting.envKey] : undefined;
			if (!this.coreRepos.settingsRepo.exists(setting.key)) {
				entries[setting.key] = envValue ?? setting.default;
			} else if (envValue && !this.coreRepos.settingsRepo.get(setting.key)) {
				// Backfill from env if the stored value is empty
				entries[setting.key] = envValue;
			}
		}
		if (Object.keys(entries).length > 0) {
			this.coreRepos.settingsRepo.setMany(entries);
		}
	}

	private buildContext(): PluginContext {
		return {
			db: this.db,
			settingsRepo: this.coreRepos.settingsRepo,
			metadataRepo: this.coreRepos.metadataRepo,
			libraryRepo: this.coreRepos.libraryRepo,
			getProcessUrl: (processId: string) => {
				return this.processes.get(processId)?.url ?? '';
			},
			isProcessAvailable: (processId: string) => {
				return this.processes.get(processId)?.available ?? false;
			},
			getRepository: <T = unknown>(localsKey: string) => {
				return this.repositories.get(localsKey) as T;
			}
		};
	}
}
