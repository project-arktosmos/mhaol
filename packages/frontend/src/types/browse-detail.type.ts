import type {
	DisplayTMDBMovie,
	DisplayTMDBTvShow,
	DisplayTMDBMovieDetails,
	DisplayTMDBTvShowDetails
} from 'addons/tmdb/types';
import type { MediaItem } from 'frontend/types/media-card.type';
import type { LibraryItemRelated } from 'frontend/types/library-item-related.type';
import type { TorrentInfo } from 'frontend/types/torrent.type';
import type {
	DisplayMusicBrainzReleaseGroup,
	DisplayMusicBrainzRelease
} from 'frontend/types/musicbrainz.type';
import type { ImageTag } from 'frontend/types/image-tagger.type';
import type { RaGameMetadata } from 'frontend/types/retroachievements.type';
import type { RightPanelVideo } from 'frontend/types/youtube.type';

export type BrowseDetailDomain = 'movie' | 'tv' | 'music' | 'photo' | 'videogame' | 'youtube' | null;

export interface PhotoImageData {
	id: string;
	libraryId: string;
	name: string;
	path: string;
	extension: string;
	tags: ImageTag[];
}

export interface BrowseDetailState {
	domain: BrowseDetailDomain;

	// Movie/TV (TMDB)
	movie: DisplayTMDBMovie | null;
	tvShow: DisplayTMDBTvShow | null;
	movieDetails: DisplayTMDBMovieDetails | null;
	tvShowDetails: DisplayTMDBTvShowDetails | null;
	libraryItem: MediaItem | null;
	relatedData: LibraryItemRelated | null;
	loading: boolean;
	fetching: boolean;
	fetched: boolean;
	downloadStatus: { state: string; progress: number } | null;
	fetchSteps: {
		terms: boolean;
		search: boolean;
		searching: boolean;
		eval: boolean;
		done: boolean;
	} | null;

	// Music
	musicAlbum: DisplayMusicBrainzReleaseGroup | null;
	musicRelease: DisplayMusicBrainzRelease | null;
	musicTorrent: TorrentInfo | null;

	// Photo
	photoImage: PhotoImageData | null;
	photoTags: string[];
	photoTagging: boolean;

	// Videogame
	videogame: RaGameMetadata | null;
	videogameDetails: RaGameMetadata | null;
	videogameDetailsLoading: boolean;

	// YouTube
	youtubeVideo: RightPanelVideo | null;
}

export interface BrowseDetailCallbacks {
	// Movie/TV
	onfetch?: () => void;
	ondownload?: () => void;
	onstream?: () => void;
	onp2pstream?: () => void;
	onshowsearch?: () => void;
	onclose?: () => void;

	// Music
	ondownloadalbum?: () => void;

	// Photo
	onaddtag?: (tag: string) => void;
	onremovetag?: (tag: string) => void;
	onautotag?: () => void;

	// YouTube
	ondownloadaudio?: () => void;
	ondownloadvideo?: () => void;
	ontogglefavorite?: () => void;
	ondeleteaudio?: () => void;
	ondeletevideo?: () => void;
}
