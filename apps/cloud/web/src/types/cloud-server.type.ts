import type { ConnectionConfig } from './connection-config.type';

export interface CloudServer extends ConnectionConfig {
	id: string;
	name: string;
	lastConnectedAt?: number;
}
