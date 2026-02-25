import { execFile } from 'node:child_process';
import { join, dirname } from 'node:path';
import { existsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import https from 'node:https';

const __dirname = dirname(fileURLToPath(import.meta.url));
const PACKAGE_ROOT = join(__dirname, '..', '..', '..');

const BOTGUARD_BIN =
	process.env.BOTGUARD_BIN ?? join(PACKAGE_ROOT, 'bin', 'rustypipe-botguard');

let cached: { visitorData: string; poToken: string; generatedAt: number } | null = null;
const TTL_MS = 6 * 60 * 60 * 1000; // 6 hours

export async function getPoToken(): Promise<{ visitorData: string; poToken: string }> {
	if (cached && Date.now() - cached.generatedAt < TTL_MS) {
		return cached;
	}
	return refreshPoToken();
}

/**
 * Generate a PO token using rustypipe-botguard (native Rust binary).
 * Fetches visitorData from YouTube, then runs the BotGuard challenge natively.
 */
export async function refreshPoToken(): Promise<{ visitorData: string; poToken: string }> {
	if (!existsSync(BOTGUARD_BIN)) {
		throw new Error(`rustypipe-botguard binary not found at ${BOTGUARD_BIN}`);
	}

	console.log('[po-token] Generating new PO token...');

	const visitorData = await fetchVisitorData();
	const poToken = await generatePoToken(visitorData);

	cached = { visitorData, poToken, generatedAt: Date.now() };
	console.log(
		`[po-token] Generated PO token (visitorData: ${visitorData.substring(0, 16)}..., token length: ${poToken.length})`
	);
	return cached;
}

export function getCachedPoToken(): {
	visitorData: string;
	poToken: string;
	generatedAt: number;
} | null {
	return cached;
}

function fetchVisitorData(): Promise<string> {
	return new Promise((resolve, reject) => {
		const url = 'https://www.youtube.com/embed/dQw4w9WgXcQ';
		const headers = {
			'user-agent':
				'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko)',
			accept: 'text/html'
		};

		https
			.get(url, { headers }, (res) => {
				let data = '';
				res.on('data', (chunk: Buffer) => {
					data += chunk.toString();
				});
				res.on('end', () => {
					const match = data.match(/"visitorData":"([^"]+)"/);
					if (match) {
						resolve(match[1]);
					} else {
						reject(new Error('Failed to extract visitorData from YouTube'));
					}
				});
			})
			.on('error', reject);
	});
}

function generatePoToken(visitorData: string): Promise<string> {
	return new Promise((resolve, reject) => {
		execFile(
			BOTGUARD_BIN,
			['--', visitorData],
			{ timeout: 60_000 },
			(error, stdout, stderr) => {
				if (error) {
					reject(
						new Error(`rustypipe-botguard failed: ${error.message}. stderr: ${stderr}`)
					);
					return;
				}

				// Output: "<token> valid_until=<ts> from_snapshot=<bool>"
				// Metadata fields match key=value pattern; the token is the first part.
				const parts = stdout.trim().split(' ');
				const token = parts[0];
				if (token && token.length > 80) {
					resolve(token);
				} else {
					reject(new Error(`rustypipe-botguard returned invalid token: ${stdout}`));
				}
			}
		);
	});
}
