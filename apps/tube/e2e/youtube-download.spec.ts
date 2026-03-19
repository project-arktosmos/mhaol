import { test, expect } from '@playwright/test';
import { existsSync, readdirSync, statSync, unlinkSync, mkdirSync } from 'node:fs';
import { join } from 'node:path';

const TEST_VIDEO_URL =
	'https://www.youtube.com/watch?v=UEcArEVH3XQ&list=RDUEcArEVH3XQ&start_radio=1';
const TEST_OUTPUT_DIR = '/tmp/mhaoltube-e2e-downloads';

function cleanOutputDir() {
	if (existsSync(TEST_OUTPUT_DIR)) {
		for (const file of readdirSync(TEST_OUTPUT_DIR)) {
			const filePath = join(TEST_OUTPUT_DIR, file);
			const stat = statSync(filePath);
			if (stat.isFile()) {
				unlinkSync(filePath);
			}
		}
	}
}

function listOutputFiles(): string[] {
	if (!existsSync(TEST_OUTPUT_DIR)) return [];
	return readdirSync(TEST_OUTPUT_DIR).filter((f) => {
		const filePath = join(TEST_OUTPUT_DIR, f);
		const stat = statSync(filePath);
		return stat.isFile() && stat.size > 0;
	});
}

test.describe('YouTube Download', () => {
	test.beforeAll(() => {
		mkdirSync(TEST_OUTPUT_DIR, { recursive: true });
		cleanOutputDir();
	});

	test.afterAll(() => {
		cleanOutputDir();
	});

	test('page loads and service initializes', async ({ page }) => {
		await page.goto('/youtube');

		await expect(page.getByRole('heading', { name: 'YouTube Download' })).toBeVisible();

		// Wait for initialization — loading spinner should disappear
		await expect(page.locator('.loading.loading-spinner')).not.toBeVisible({ timeout: 15_000 });

		// yt-dlp status should show ready
		await expect(page.getByText('yt-dlp Ready')).toBeVisible({ timeout: 10_000 });
	});

	test('invalid URL shows validation error', async ({ page }) => {
		await page.goto('/youtube');
		await expect(page.getByText('yt-dlp Ready')).toBeVisible({ timeout: 15_000 });

		const urlInput = page.locator('input[placeholder*="youtube.com/watch"]');
		await urlInput.fill('not-a-valid-url');

		await expect(page.getByText('Please enter a valid YouTube URL')).toBeVisible();

		// Fetch button should be disabled
		const fetchButton = page.locator('.join button:has-text("Fetch")');
		await expect(fetchButton).toBeDisabled();
	});

	test('fetches video info when URL is submitted', async ({ page }) => {
		await page.goto('/youtube');
		await expect(page.getByText('yt-dlp Ready')).toBeVisible({ timeout: 15_000 });

		const urlInput = page.locator('input[placeholder*="youtube.com/watch"]');
		await urlInput.fill(TEST_VIDEO_URL);

		const fetchButton = page.locator('.join button:has-text("Fetch")');
		await fetchButton.click();

		// Wait for video preview to appear — the title in YouTubeVideoPreview
		const videoTitle = page.locator('.card-body .text-lg.font-semibold');
		await expect(videoTitle).toBeVisible({ timeout: 30_000 });
		await expect(videoTitle).not.toBeEmpty();

		// Duration badge should be visible
		await expect(page.locator('.card-body .badge-ghost').first()).toBeVisible();
	});

	test('downloads audio end-to-end', async ({ page }) => {
		await page.goto('/youtube');
		await expect(page.getByText('yt-dlp Ready')).toBeVisible({ timeout: 15_000 });

		// Set output path and audio mode via API
		await page.evaluate(async (outputDir: string) => {
			await fetch('/api/ytdl/settings', {
				method: 'PUT',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ outputPath: outputDir, downloadMode: 'audio' })
			});
		}, TEST_OUTPUT_DIR);

		await page.reload();
		await expect(page.getByText('yt-dlp Ready')).toBeVisible({ timeout: 15_000 });

		// Clear previous downloads
		await page.evaluate(async () => {
			await fetch('/api/ytdl/downloads/completed', { method: 'DELETE' });
		});

		// Enter URL and fetch info
		const urlInput = page.locator('input[placeholder*="youtube.com/watch"]');
		await urlInput.fill(TEST_VIDEO_URL);
		await page.locator('.join button:has-text("Fetch")').click();

		// Wait for video preview
		await expect(page.locator('.card-body .text-lg.font-semibold')).toBeVisible({
			timeout: 30_000
		});

		// Click Download
		await page.locator('.card-body .btn-primary:has-text("Download")').click();

		// Wait for completion
		const downloadQueue = page.locator('.card-body:has(.card-title:has-text("Downloads"))');
		await expect(downloadQueue.locator('.badge-sm').first()).toBeVisible({ timeout: 10_000 });

		const completedBadge = downloadQueue.getByText('Completed');
		const failedBadge = downloadQueue.getByText('Failed');
		await expect(completedBadge.or(failedBadge)).toBeVisible({ timeout: 120_000 });

		if (await failedBadge.isVisible().catch(() => false)) {
			const errorText = await downloadQueue.locator('.text-error').textContent();
			throw new Error(`Audio download failed: ${errorText}`);
		}

		// Verify file on disk
		await page.waitForTimeout(3_000);
		const files = listOutputFiles();
		console.log(`Audio download files:`, files);
		expect(files.length).toBeGreaterThan(0);

		const audioFile = files.find(
			(f) =>
				f.endsWith('.m4a') ||
				f.endsWith('.mp3') ||
				f.endsWith('.opus') ||
				f.endsWith('.mp4') ||
				f.endsWith('.webm')
		);
		expect(audioFile).toBeDefined();
	});

	test('downloads video end-to-end', async ({ page }) => {
		cleanOutputDir();

		await page.goto('/youtube');
		await expect(page.getByText('yt-dlp Ready')).toBeVisible({ timeout: 15_000 });

		// Set output path and video mode via API
		await page.evaluate(async (outputDir: string) => {
			await fetch('/api/ytdl/settings', {
				method: 'PUT',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ outputPath: outputDir, downloadMode: 'video' })
			});
		}, TEST_OUTPUT_DIR);

		await page.reload();
		await expect(page.getByText('yt-dlp Ready')).toBeVisible({ timeout: 15_000 });

		// Clear previous downloads
		await page.evaluate(async () => {
			await fetch('/api/ytdl/downloads/completed', { method: 'DELETE' });
		});

		// Enter URL and fetch info
		const urlInput = page.locator('input[placeholder*="youtube.com/watch"]');
		await urlInput.fill(TEST_VIDEO_URL);
		await page.locator('.join button:has-text("Fetch")').click();

		// Wait for video preview
		await expect(page.locator('.card-body .text-lg.font-semibold')).toBeVisible({
			timeout: 30_000
		});

		// Click Download
		await page.locator('.card-body .btn-primary:has-text("Download")').click();

		// Wait for completion
		const downloadQueue = page.locator('.card-body:has(.card-title:has-text("Downloads"))');
		await expect(downloadQueue.locator('.badge-sm').first()).toBeVisible({ timeout: 10_000 });

		const completedBadge = downloadQueue.getByText('Completed');
		const failedBadge = downloadQueue.getByText('Failed');
		await expect(completedBadge.or(failedBadge)).toBeVisible({ timeout: 120_000 });

		if (await failedBadge.isVisible().catch(() => false)) {
			const errorText = await downloadQueue.locator('.text-error').textContent();
			throw new Error(`Video download failed: ${errorText}`);
		}

		// Verify file on disk
		await page.waitForTimeout(3_000);
		const files = listOutputFiles();
		console.log(`Video download files:`, files);
		expect(files.length).toBeGreaterThan(0);

		const videoFile = files.find(
			(f) => f.endsWith('.mp4') || f.endsWith('.mkv') || f.endsWith('.webm')
		);
		expect(videoFile).toBeDefined();
	});
});
