export interface HubApp {
	name: string;
	port: number;
	status: 'starting' | 'running' | 'stopped' | 'failed' | 'unknown';
	has_headless: boolean;
	logs: string[];
}
