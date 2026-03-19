import { AdapterClass } from 'frontend/adapters/classes/adapter.class';
import type { Library, LibraryFile } from 'frontend/types/library.type';
import { LibraryType } from 'frontend/types/library.type';
import type { PeerLibrarySummary, PeerLibraryFileInfo } from 'frontend/types/peer-library.type';

export class PeerLibraryAdapter extends AdapterClass {
	constructor() {
		super('peer-library');
	}

	toSummaries(
		libraries: Library[],
		libraryFiles: Record<string, LibraryFile[]>
	): PeerLibrarySummary[] {
		return libraries.map((lib) => ({
			id: lib.id as string,
			name: lib.name,
			libraryType: lib.libraryType,
			fileCount: (libraryFiles[lib.id as string] ?? []).length
		}));
	}

	toFileInfos(files: LibraryFile[]): PeerLibraryFileInfo[] {
		return files.map((f) => ({
			id: f.id,
			name: f.name,
			extension: f.extension,
			mediaType: f.mediaType
		}));
	}

	libraryTypeLabel(type: LibraryType): string {
		switch (type) {
			case LibraryType.Movies:
				return 'Movies';
			case LibraryType.TV:
				return 'TV Shows';
			default:
				return type;
		}
	}
}

export const peerLibraryAdapter = new PeerLibraryAdapter();
