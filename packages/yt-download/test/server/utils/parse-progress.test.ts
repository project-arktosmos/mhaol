import { describe, it, expect } from 'vitest';
import {
	parseProgressPercent,
	parseDestination,
	parseExtractAudioDestination,
	parseAlreadyDownloaded
} from '../../../src/server/utils/parse-progress.js';

describe('parseProgressPercent', () => {
	it('parses percentage from typical yt-dlp output', () => {
		expect(parseProgressPercent('[download]  50.0% of 5.00MiB at 1.00MiB/s')).toBe(50.0);
	});

	it('parses 100%', () => {
		expect(parseProgressPercent('[download] 100% of 5.00MiB')).toBe(100);
	});

	it('parses 0%', () => {
		expect(parseProgressPercent('[download]   0.0% of 5.00MiB')).toBe(0.0);
	});

	it('parses decimal percentages', () => {
		expect(parseProgressPercent('[download]  73.5% of 10.00MiB at 2.50MiB/s')).toBe(73.5);
	});

	it('returns null for lines without percentage', () => {
		expect(parseProgressPercent('[download] Destination: /path/to/file.m4a')).toBeNull();
	});

	it('returns null for empty string', () => {
		expect(parseProgressPercent('')).toBeNull();
	});
});

describe('parseDestination', () => {
	it('parses download destination line', () => {
		expect(
			parseDestination('[download] Destination: /tmp/downloads/My Song.webm')
		).toBe('/tmp/downloads/My Song.webm');
	});

	it('returns null for non-destination lines', () => {
		expect(parseDestination('[download]  50.0% of 5.00MiB')).toBeNull();
	});
});

describe('parseExtractAudioDestination', () => {
	it('parses extract audio destination line', () => {
		expect(
			parseExtractAudioDestination('[ExtractAudio] Destination: /tmp/downloads/My Song.m4a')
		).toBe('/tmp/downloads/My Song.m4a');
	});

	it('returns null for non-extract lines', () => {
		expect(parseExtractAudioDestination('[download] Destination: /tmp/file.webm')).toBeNull();
	});
});

describe('parseAlreadyDownloaded', () => {
	it('parses already downloaded line', () => {
		expect(
			parseAlreadyDownloaded('[download] [/tmp/downloads/My Song.m4a] has already been downloaded')
		).toBe('/tmp/downloads/My Song.m4a');
	});

	it('returns null for non-matching lines', () => {
		expect(parseAlreadyDownloaded('[download] 50% of 5.00MiB')).toBeNull();
	});
});
