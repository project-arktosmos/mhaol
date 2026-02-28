import { test, expect } from '@playwright/test';

test.describe('Player Audio Streaming', () => {
	test('page loads and initializes', async ({ page }) => {
		await page.goto('/player');

		await expect(page.getByRole('heading', { name: 'Player' })).toBeVisible();

		// Wait for initialization — loading spinner should disappear
		await expect(page.locator('.loading.loading-spinner')).not.toBeVisible({ timeout: 15_000 });
	});

	test('shows stream server status', async ({ page }) => {
		await page.goto('/player');
		await expect(page.getByRole('heading', { name: 'Player' })).toBeVisible();

		// Should show either "Stream server ready" or "Stream server offline"
		const ready = page.getByText('Stream server ready');
		const offline = page.getByText('Stream server offline');
		await expect(ready.or(offline)).toBeVisible({ timeout: 15_000 });
	});

	test('filter buttons work for audio files', async ({ page }) => {
		await page.goto('/player');
		await expect(page.getByRole('heading', { name: 'Player' })).toBeVisible();
		await expect(page.locator('.loading.loading-spinner')).not.toBeVisible({ timeout: 15_000 });

		// Click the Audio filter button
		const audioFilterBtn = page.locator('.join button:has-text("Audio")');
		await audioFilterBtn.click();
		await expect(audioFilterBtn).toHaveClass(/btn-active/);

		// Click the Video filter button
		const videoFilterBtn = page.locator('.join button:has-text("Video")');
		await videoFilterBtn.click();
		await expect(videoFilterBtn).toHaveClass(/btn-active/);

		// Click the All filter button
		const allFilterBtn = page.locator('.join button:has-text("All")');
		await allFilterBtn.click();
		await expect(allFilterBtn).toHaveClass(/btn-active/);
	});

	test('streams an audio file end-to-end', async ({ page }) => {
		await page.goto('/player');
		await expect(page.getByRole('heading', { name: 'Player' })).toBeVisible();
		await expect(page.locator('.loading.loading-spinner')).not.toBeVisible({ timeout: 15_000 });

		// Stream server must be available
		await expect(page.getByText('Stream server ready')).toBeVisible({ timeout: 15_000 });

		// Filter to audio files
		const audioFilterBtn = page.locator('.join button:has-text("Audio")');
		await audioFilterBtn.click();
		await expect(audioFilterBtn).toHaveClass(/btn-active/);

		// Check if there are audio files available
		const noFiles = page.getByText('No playable files found.');
		const firstAudioFile = page.locator('.card-body button.flex.w-full').first();
		const hasFiles = await firstAudioFile.isVisible({ timeout: 5_000 }).catch(() => false);

		if (!hasFiles) {
			// Skip if no audio files are available — test environment may not have downloads
			test.skip(true, 'No audio files available in player library');
			return;
		}

		// Get the file name for later verification
		const fileName = await firstAudioFile.locator('.text-sm.font-medium').textContent();
		expect(fileName).toBeTruthy();

		// Click the audio file to start streaming
		await firstAudioFile.click();

		// The "Now Playing" modal should appear
		const modal = page.locator('.modal.modal-open');
		await expect(modal).toBeVisible({ timeout: 10_000 });
		await expect(modal.getByText('Now Playing')).toBeVisible();

		// File name should appear in the modal
		await expect(modal.locator('.text-sm.opacity-60')).toContainText(fileName!.trim());

		// Should show connecting/signaling state (spinner + status text)
		// Then transition to streaming
		const streamingControls = modal.locator('[aria-label="Play"], [aria-label="Pause"]');
		await expect(streamingControls.first()).toBeVisible({ timeout: 30_000 });

		// Audio element should exist (rendered off-screen, not hidden)
		// PlayerVideo renders <audio> for audio mode with class "absolute h-0 w-0 overflow-hidden"
		const audioElement = modal.locator('audio');
		await expect(audioElement).toBeAttached({ timeout: 10_000 });

		// Verify the audio element has a srcObject (MediaStream attached)
		const hasSrcObject = await page.evaluate(() => {
			const audio = document.querySelector('.modal.modal-open audio');
			return audio && (audio as HTMLAudioElement).srcObject !== null;
		});
		expect(hasSrcObject).toBe(true);

		// Player controls should be visible: play/pause, volume, stop
		await expect(modal.locator('[aria-label="Stop"]')).toBeVisible();
		await expect(modal.locator('[aria-label="Mute"], [aria-label="Unmute"]')).toBeVisible();

		// No fullscreen button for audio-only
		await expect(modal.locator('[aria-label="Fullscreen"]')).not.toBeVisible();

		// No error alerts should appear
		await expect(page.locator('.alert.alert-error')).not.toBeVisible();

		// Stop playback via the close button
		const closeBtn = modal.locator('button:has-text("×")');
		await closeBtn.click();

		// Modal should close
		await expect(modal).not.toBeVisible({ timeout: 5_000 });

		// Status should return to ready
		await expect(page.getByText('Stream server ready')).toBeVisible({ timeout: 10_000 });
	});

	test('streams a video file end-to-end (regression check)', async ({ page }) => {
		await page.goto('/player');
		await expect(page.getByRole('heading', { name: 'Player' })).toBeVisible();
		await expect(page.locator('.loading.loading-spinner')).not.toBeVisible({ timeout: 15_000 });

		// Stream server must be available
		await expect(page.getByText('Stream server ready')).toBeVisible({ timeout: 15_000 });

		// Filter to video files
		const videoFilterBtn = page.locator('.join button:has-text("Video")');
		await videoFilterBtn.click();
		await expect(videoFilterBtn).toHaveClass(/btn-active/);

		// Check if there are video files available
		const firstVideoFile = page.locator('.card-body button.flex.w-full').first();
		const hasFiles = await firstVideoFile.isVisible({ timeout: 5_000 }).catch(() => false);

		if (!hasFiles) {
			test.skip(true, 'No video files available in player library');
			return;
		}

		// Click the video file to start streaming
		await firstVideoFile.click();

		// The "Now Playing" modal should appear
		const modal = page.locator('.modal.modal-open');
		await expect(modal).toBeVisible({ timeout: 10_000 });

		// Should transition to streaming with controls visible
		const streamingControls = modal.locator('[aria-label="Play"], [aria-label="Pause"]');
		await expect(streamingControls.first()).toBeVisible({ timeout: 30_000 });

		// Video element should exist with srcObject
		const videoElement = modal.locator('video');
		await expect(videoElement).toBeAttached({ timeout: 10_000 });

		const hasSrcObject = await page.evaluate(() => {
			const video = document.querySelector('.modal.modal-open video');
			return video && (video as HTMLVideoElement).srcObject !== null;
		});
		expect(hasSrcObject).toBe(true);

		// Fullscreen button should be visible for video
		await expect(modal.locator('[aria-label="Fullscreen"]')).toBeVisible();

		// No error alerts
		await expect(page.locator('.alert.alert-error')).not.toBeVisible();

		// Stop playback
		const closeBtn = modal.locator('button:has-text("×")');
		await closeBtn.click();
		await expect(modal).not.toBeVisible({ timeout: 5_000 });
	});
});
