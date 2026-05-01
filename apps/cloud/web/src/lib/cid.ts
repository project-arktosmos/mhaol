const BASE32_ALPHABET = 'abcdefghijklmnopqrstuvwxyz234567';

function base32EncodeLower(bytes: Uint8Array): string {
	let bits = 0;
	let value = 0;
	let output = '';
	for (let i = 0; i < bytes.length; i++) {
		value = (value << 8) | bytes[i];
		bits += 8;
		while (bits >= 5) {
			output += BASE32_ALPHABET[(value >>> (bits - 5)) & 0x1f];
			bits -= 5;
		}
	}
	if (bits > 0) {
		output += BASE32_ALPHABET[(value << (5 - bits)) & 0x1f];
	}
	return output;
}

export async function computeCidV1Raw(bytes: Uint8Array): Promise<string> {
	const buffer = new ArrayBuffer(bytes.byteLength);
	new Uint8Array(buffer).set(bytes);
	const digest = new Uint8Array(await crypto.subtle.digest('SHA-256', buffer));
	const cidBytes = new Uint8Array(4 + digest.length);
	cidBytes[0] = 0x01;
	cidBytes[1] = 0x55;
	cidBytes[2] = 0x12;
	cidBytes[3] = 0x20;
	cidBytes.set(digest, 4);
	return 'b' + base32EncodeLower(cidBytes);
}
