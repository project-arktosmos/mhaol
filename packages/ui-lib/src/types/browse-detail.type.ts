import type {
	DisplayTMDBMovie,
	DisplayTMDBTvShow,
	DisplayTMDBMovieDetails,
	DisplayTMDBTvShowDetails,
	DisplayTMDBSeasonDetails
} from 'addons/tmdb/types';
import type { MediaItem } from 'ui-lib/types/media-card.type';
import type { LibraryItemRelated } from 'ui-lib/types/library-item-related.type';
import type { TorrentInfo } from 'ui-lib/types/torrent.type';
import type {
	DisplayMusicBrainzReleaseGroup,
	DisplayMusicBrainzRelease
} from 'addons/musicbrainz/types';
import type { ImageTag } from 'ui-lib/types/image-tagger.type';
import type { RaGameMetadata } from 'addons/retroachievements/types';
import type { RightPanelVideo } from 'ui-lib/types/youtube.type';
import type { DisplayBook, DisplayBookDetails } from 'addons/openlibrary/types';

export type BrowseDetailDomain =
	| 'movie'
	| 'tv'
	| 'music'
	| 'photo'
	| 'videogame'
	| 'youtube'
	| 'book'
	| null;

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
	tvSeasonDetails: DisplayTMDBSeasonDetails[];
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

	// Book
	book: DisplayBook | null;
	bookDetails: DisplayBookDetails | null;
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
