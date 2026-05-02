/// Deterministic 6-char hex color from an arbitrary string. Uses FNV-1a 32-bit
/// hash, masks to 24 bits, and pads to a 6-char lowercase hex string suitable
/// for use as `#${hashColor(...)}` in CSS.
export function hashColor(input: string): string {
	let hash = 0x811c9dc5;
	for (let i = 0; i < input.length; i++) {
		hash ^= input.charCodeAt(i);
		hash = Math.imul(hash, 0x01000193) >>> 0;
	}
	return (hash & 0xffffff).toString(16).padStart(6, '0');
}
