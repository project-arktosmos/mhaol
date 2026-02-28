import { readFileSync, writeFileSync, existsSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { generatePrivateKey, privateKeyToAccount } from 'viem/accounts';
import { hashMessage } from 'viem';

function findRepoRoot(): string {
	let dir = dirname(new URL(import.meta.url).pathname);
	while (dir !== '/') {
		if (existsSync(join(dir, 'pnpm-workspace.yaml'))) return dir;
		dir = dirname(dir);
	}
	return process.cwd();
}

const IDENTITIES_FILE = join(findRepoRoot(), '.env.identities');

function parse(): Record<string, string> {
	if (!existsSync(IDENTITIES_FILE)) return {};

	const content = readFileSync(IDENTITIES_FILE, 'utf-8');
	const entries: Record<string, string> = {};

	for (const line of content.split('\n')) {
		const trimmed = line.trim();
		if (!trimmed || trimmed.startsWith('#')) continue;

		const eqIndex = trimmed.indexOf('=');
		if (eqIndex === -1) continue;

		const key = trimmed.slice(0, eqIndex).trim();
		const value = trimmed.slice(eqIndex + 1).trim();
		if (key && value) entries[key] = value;
	}

	return entries;
}

function write(entries: Record<string, string>): void {
	const lines = Object.entries(entries).map(([key, value]) => `${key}=${value}`);
	writeFileSync(IDENTITIES_FILE, lines.join('\n') + '\n', 'utf-8');
}

export function getAll(): Record<string, string> {
	const entries = parse();
	const result: Record<string, string> = {};

	for (const [name, privateKey] of Object.entries(entries)) {
		const account = privateKeyToAccount(privateKey as `0x${string}`);
		result[name] = account.address;
	}

	return result;
}

export function getAddress(name: string): string | null {
	const entries = parse();
	const privateKey = entries[name];
	if (!privateKey) return null;

	const account = privateKeyToAccount(privateKey as `0x${string}`);
	return account.address;
}

export function getPrivateKey(name: string): string | null {
	const entries = parse();
	return entries[name] ?? null;
}

export function set(name: string, privateKey: string): string {
	const entries = parse();
	entries[name] = privateKey;
	write(entries);

	const account = privateKeyToAccount(privateKey as `0x${string}`);
	return account.address;
}

export function regenerate(name: string): string {
	const privateKey = generatePrivateKey();
	return set(name, privateKey);
}

export function remove(name: string): boolean {
	const entries = parse();
	if (!(name in entries)) return false;
	delete entries[name];
	write(entries);
	return true;
}

export async function getPassport(
	name: string
): Promise<{ raw: string; hash: string; signature: string } | null> {
	const entries = parse();
	const privateKey = entries[name];
	if (!privateKey) return null;

	const account = privateKeyToAccount(privateKey as `0x${string}`);
	const raw = JSON.stringify({ name, address: account.address });
	const hash = hashMessage(raw);
	const signature = await account.signMessage({ message: raw });

	return { raw, hash, signature };
}

export function ensureIdentity(name: string): string {
	const address = getAddress(name);
	if (address) return address;
	return regenerate(name);
}

export function getDefaultAddress(): string | null {
	const entries = parse();
	const firstKey = Object.keys(entries)[0];
	if (!firstKey) return null;
	const account = privateKeyToAccount(entries[firstKey] as `0x${string}`);
	return account.address;
}
