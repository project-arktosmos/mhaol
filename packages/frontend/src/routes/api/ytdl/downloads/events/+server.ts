import type { RequestHandler } from './$types';

export const GET: RequestHandler = async ({ locals }) => {
	const res = await fetch(`${locals.ytdlBaseUrl}/api/downloads/events`, {
		headers: { Accept: 'text/event-stream' }
	}).catch(() => null);

	if (!res || !res.ok || !res.body) {
		return new Response('YouTube download server not available', { status: 503 });
	}

	const reader = res.body.getReader();
	const decoder = new TextDecoder();
	const youtubeDownloadRepo = locals.youtubeDownloadRepo;

	const stream = new ReadableStream({
		async start(controller) {
			const encoder = new TextEncoder();
			let buffer = '';

			try {
				while (true) {
					const { done, value } = await reader.read();
					if (done) break;

					buffer += decoder.decode(value, { stream: true });

					const lines = buffer.split('\n');
					buffer = lines.pop() ?? '';

					let currentEvent = '';
					let currentData = '';

					for (const line of lines) {
						if (line.startsWith('event:')) {
							currentEvent = line.slice(6).trim();
						} else if (line.startsWith('data:')) {
							currentData = line.slice(5).trim();
						} else if (line === '' && currentEvent && currentData) {
							controller.enqueue(
								encoder.encode(`event: ${currentEvent}\ndata: ${currentData}\n\n`)
							);

							if (currentEvent === 'progress') {
								try {
									const progress = JSON.parse(currentData);
									youtubeDownloadRepo.upsert({
										download_id: progress.downloadId,
										url: progress.url,
										video_id: progress.videoId,
										title: progress.title,
										state: progress.state,
										progress: progress.progress,
										downloaded_bytes: progress.downloadedBytes,
										total_bytes: progress.totalBytes,
										output_path: progress.outputPath,
										error: progress.error,
										mode: progress.mode,
										quality: progress.quality,
										format: progress.format,
										video_quality: progress.videoQuality,
										video_format: progress.videoFormat,
										thumbnail_url: progress.thumbnailUrl,
										duration_seconds: progress.durationSeconds
									});
								} catch {
									// ignore parse/persist errors
								}
							}

							currentEvent = '';
							currentData = '';
						}
					}
				}
			} catch {
				// connection closed
			} finally {
				controller.close();
			}
		},

		cancel() {
			reader.cancel();
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
