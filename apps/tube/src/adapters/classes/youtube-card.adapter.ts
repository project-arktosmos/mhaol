import { AdapterClass } from '$adapters/classes/adapter.class';
import type { LibraryCardItem } from '$types/library.type';
import type { YouTubeContent, YouTubeRssVideo } from '$types/youtube.type';
import type { YouTubeSearchItem } from '$types/youtube-search.type';

export class YouTubeCardAdapter extends AdapterClass {
	constructor() {
		super('youtube-card');
	}

	fromContent(item: YouTubeContent): LibraryCardItem {
		return {
			videoId: item.youtubeId,
			title: item.title,
			thumbnailUrl: item.thumbnailUrl,
			durationSeconds: item.durationSeconds,
			channelName: item.channelName,
			hasVideo: item.hasVideo,
			hasAudio: item.hasAudio
		};
	}

	fromRssVideo(video: YouTubeRssVideo): LibraryCardItem {
		return {
			videoId: video.videoId,
			title: video.title,
			thumbnailUrl: video.thumbnail,
			durationSeconds: null,
			channelName: null,
			hasVideo: false,
			hasAudio: false
		};
	}

	fromSearchItem(item: YouTubeSearchItem): LibraryCardItem {
		return {
			videoId: item.videoId,
			title: item.title,
			thumbnailUrl: item.thumbnail,
			durationSeconds: item.duration > 0 ? item.duration : null,
			channelName: item.uploaderName || null,
			hasVideo: false,
			hasAudio: false
		};
	}
}

export const youTubeCardAdapter = new YouTubeCardAdapter();
