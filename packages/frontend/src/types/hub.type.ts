export interface HubApp {
	name: string;
	port: number;
	status: 'building' | 'starting' | 'running' | 'stopped' | 'failed' | 'unknown';
	has_headless: boolean;
	frontend_built: boolean;
	backend_built: boolean;
	build_logs: string[];
	runtime_logs: string[];
}
