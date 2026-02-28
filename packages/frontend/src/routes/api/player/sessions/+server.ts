import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const POST: RequestHandler = async ({ request, locals }) => {
	const bridge = locals.p2pWorkerBridge;
	if (!bridge?.isAvailable()) {
		return json({ error: 'Streaming worker is not available' }, { status: 503 });
	}

	const body = await request.json();
	const sessionId = crypto.randomUUID();

	// Use dev signaling URL or deployed PartyKit URL
	const signalingUrl = locals.signalingDevUrl || locals.signalingPartyUrl;
	if (!signalingUrl) {
		return json({ error: 'No signaling server available' }, { status: 503 });
	}

	try {
		const { room_id } = await bridge.createSession({
			sessionId,
			filePath: body.file_path,
			signalingUrl,
			mode: body.mode,
			videoCodec: body.video_codec,
			videoQuality: body.video_quality
		});

		return json({
			session_id: sessionId,
			room_id,
			signaling_url: signalingUrl
		});
	} catch (e) {
		const message = e instanceof Error ? e.message : 'Unknown error';
		return json({ error: message }, { status: 500 });
	}
};
