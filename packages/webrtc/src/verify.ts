import { recoverMessageAddress } from 'viem';
import type { PassportData, PassportPayload } from 'webrtc/types';

export async function verifyPassport(passport: PassportData): Promise<PassportPayload> {
	const payload: PassportPayload = JSON.parse(passport.raw);

	if (!payload.name || !payload.address) {
		throw new Error('Passport payload missing name or address');
	}

	const recoveredAddress = await recoverMessageAddress({
		message: passport.raw,
		signature: passport.signature as `0x${string}`
	});

	if (recoveredAddress.toLowerCase() !== payload.address.toLowerCase()) {
		throw new Error(
			`Passport signature mismatch: claimed ${payload.address}, recovered ${recoveredAddress}`
		);
	}

	return payload;
}
