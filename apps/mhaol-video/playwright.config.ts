import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
	testDir: './e2e',
	fullyParallel: false,
	forbidOnly: !!process.env.CI,
	retries: 0,
	workers: 1,
	reporter: 'html',
	timeout: 180_000,
	expect: { timeout: 30_000 },

	use: {
		baseURL: process.env.E2E_BASE_URL ?? 'http://localhost:1530',
		trace: 'on-first-retry',
		screenshot: 'only-on-failure',
		video: 'retain-on-failure'
	},

	projects: [
		{
			name: 'chromium',
			use: { ...devices['Desktop Chrome'] }
		}
	],

	webServer: {
		command: `pnpm dev${process.env.E2E_PORT ? ' -- --port ' + process.env.E2E_PORT : ''}`,
		url: process.env.E2E_BASE_URL ?? 'http://localhost:1530',
		reuseExistingServer: true,
		timeout: 120_000,
		cwd: '../../'
	}
});
