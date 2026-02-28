import { spawn, type ChildProcess } from 'node:child_process';
import { createInterface, type Interface } from 'node:readline';

interface WorkerEvent {
	event: string;
	session_id?: string;
	room_id?: string;
	error?: string;
}

interface PendingRequest {
	resolve: (event: WorkerEvent) => void;
	reject: (error: Error) => void;
	timer: ReturnType<typeof setTimeout>;
}

/**
 * Manages a Rust stdio worker process.
 *
 * Communication:
 * - Commands go to the worker via stdin (JSON lines)
 * - Events come back via stdout (JSON lines)
 * - Tracing/logs go to stderr (forwarded with prefix)
 */
export class WorkerBridge {
	private process: ChildProcess | null = null;
	private readline: Interface | null = null;
	private pending: Map<string, PendingRequest> = new Map();
	private logPrefix: string;

	constructor(logPrefix: string = 'worker') {
		this.logPrefix = logPrefix;
	}

	/**
	 * Spawn the worker process.
	 */
	start(binaryPath: string, env?: Record<string, string>): void {
		if (this.process) return;

		this.process = spawn(binaryPath, [], {
			stdio: ['pipe', 'pipe', 'pipe'],
			env: { ...process.env, ...env }
		});

		// Read stdout JSON lines (protocol events)
		this.readline = createInterface({ input: this.process.stdout! });
		this.readline.on('line', (line) => this.handleEvent(line));

		// Forward stderr as logs
		this.process.stderr?.on('data', (data: Buffer) => {
			for (const line of data.toString().trimEnd().split('\n')) {
				console.error(`[${this.logPrefix}] ${line}`);
			}
		});

		this.process.on('error', (err) => {
			console.error(`[${this.logPrefix}] Failed to start: ${err.message}`);
			this.process = null;
			this.rejectAll('Worker process failed to start');
		});

		this.process.on('exit', (code) => {
			console.log(`[${this.logPrefix}] Process exited with code ${code}`);
			this.process = null;
			this.readline = null;
			this.rejectAll('Worker process exited');
		});

		console.log(`[${this.logPrefix}] Started (pid: ${this.process.pid})`);
	}

	/**
	 * Create a streaming session. Returns the room ID for PartyKit signaling.
	 */
	async createSession(params: {
		sessionId: string;
		filePath: string;
		signalingUrl: string;
		mode?: string;
		videoCodec?: string;
		videoQuality?: string;
	}): Promise<{ room_id: string }> {
		const command = {
			command: 'create_session',
			session_id: params.sessionId,
			file_path: params.filePath,
			signaling_url: params.signalingUrl,
			mode: params.mode,
			video_codec: params.videoCodec,
			video_quality: params.videoQuality
		};

		const event = await this.sendCommand(params.sessionId, command);

		if (event.event === 'error') {
			throw new Error(event.error ?? 'Unknown error');
		}

		return { room_id: event.room_id! };
	}

	/**
	 * Delete a streaming session.
	 */
	async deleteSession(sessionId: string): Promise<void> {
		const command = {
			command: 'delete_session',
			session_id: sessionId
		};

		const event = await this.sendCommand(sessionId, command);

		if (event.event === 'error') {
			throw new Error(event.error ?? 'Unknown error');
		}
	}

	/**
	 * Check if the worker process is running.
	 */
	isAvailable(): boolean {
		return this.process !== null && this.process.exitCode === null;
	}

	/**
	 * Kill the worker process.
	 */
	shutdown(): void {
		if (this.process) {
			this.process.stdin?.end();
			this.process.kill();
			this.process = null;
		}
		this.readline = null;
		this.rejectAll('Worker shutting down');
	}

	private sendCommand(sessionId: string, command: Record<string, unknown>): Promise<WorkerEvent> {
		return new Promise((resolve, reject) => {
			if (!this.process?.stdin?.writable) {
				reject(new Error('Worker process not available'));
				return;
			}

			const timer = setTimeout(() => {
				this.pending.delete(sessionId);
				reject(new Error('Worker command timed out'));
			}, 30_000);

			this.pending.set(sessionId, { resolve, reject, timer });

			const json = JSON.stringify(command) + '\n';
			this.process.stdin.write(json);
		});
	}

	private handleEvent(line: string): void {
		let event: WorkerEvent;
		try {
			event = JSON.parse(line);
		} catch {
			console.warn(`[${this.logPrefix}] Invalid JSON on stdout: ${line}`);
			return;
		}

		const sessionId = event.session_id;
		if (sessionId && this.pending.has(sessionId)) {
			const pending = this.pending.get(sessionId)!;
			this.pending.delete(sessionId);
			clearTimeout(pending.timer);
			pending.resolve(event);
		}
	}

	private rejectAll(reason: string): void {
		for (const [id, pending] of this.pending) {
			clearTimeout(pending.timer);
			pending.reject(new Error(reason));
		}
		this.pending.clear();
	}
}
