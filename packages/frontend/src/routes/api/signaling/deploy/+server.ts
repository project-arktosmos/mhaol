import { spawn } from 'node:child_process';
import { existsSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { getDefaultAddress } from '$lib/server/identities';
import type { RequestHandler } from './$types';

const __dirname = dirname(fileURLToPath(import.meta.url));
const FRONTEND_ROOT = join(__dirname, '..', '..', '..', '..', '..');
const SIGNALING_DIR = join(FRONTEND_ROOT, '..', 'signaling');

let deploying = false;

function extractDeployedUrl(output: string): string | null {
	const match = output.match(/https:\/\/[\w-]+\.[\w.-]*partykit\.dev/);
	return match ? match[0] : null;
}

export const HEAD: RequestHandler = async () => {
	return new Response(null, {
		status: deploying ? 409 : 200
	});
};

export const GET: RequestHandler = async ({ request }) => {
	if (deploying) {
		return new Response(JSON.stringify({ error: 'Deploy already in progress' }), {
			status: 409,
			headers: { 'Content-Type': 'application/json' }
		});
	}

	if (!existsSync(SIGNALING_DIR)) {
		return new Response(
			JSON.stringify({ error: `Signaling package not found at ${SIGNALING_DIR}` }),
			{ status: 500, headers: { 'Content-Type': 'application/json' } }
		);
	}

	deploying = true;

	const address = getDefaultAddress();
	if (!address) {
		deploying = false;
		return new Response(
			JSON.stringify({ error: 'No identity found. Create one in the Identity page first.' }),
			{ status: 400, headers: { 'Content-Type': 'application/json' } }
		);
	}
	const shortAddr = address.toLowerCase().slice(2, 10);
	const deployName = `${shortAddr}-mhaol-signaling`;

	const child = spawn('npx', ['partykit', 'deploy', '--name', deployName], {
		cwd: SIGNALING_DIR,
		env: { ...process.env },
		stdio: ['ignore', 'pipe', 'pipe']
	});

	request.signal.addEventListener('abort', () => {
		child.kill();
		deploying = false;
	});

	const stream = new ReadableStream({
		start(controller) {
			const encoder = new TextEncoder();

			const send = (event: string, data: unknown) => {
				controller.enqueue(encoder.encode(`event: ${event}\ndata: ${JSON.stringify(data)}\n\n`));
			};

			let allOutput = '';

			child.stdout?.on('data', (chunk: Buffer) => {
				const text = chunk.toString();
				allOutput += text;
				for (const line of text.split('\n').filter(Boolean)) {
					send('log', { stream: 'stdout', text: line });
				}
			});

			child.stderr?.on('data', (chunk: Buffer) => {
				const text = chunk.toString();
				allOutput += text;
				for (const line of text.split('\n').filter(Boolean)) {
					send('log', { stream: 'stderr', text: line });
				}
			});

			child.on('error', (err) => {
				send('error', { message: err.message });
				deploying = false;
				controller.close();
			});

			child.on('close', (code) => {
				const success = code === 0;
				const url = extractDeployedUrl(allOutput);
				send('done', { success, code, ...(url ? { url } : {}) });
				deploying = false;
				controller.close();
			});
		},
		cancel() {
			child.kill();
			deploying = false;
		}
	});

	return new Response(stream, {
		headers: {
			'Content-Type': 'text/event-stream',
			'Cache-Control': 'no-cache',
			Connection: 'keep-alive'
		}
	});
};
