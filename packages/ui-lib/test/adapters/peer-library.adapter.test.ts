import { describe, it, expect } from 'vitest';
import { peerLibraryAdapter } from '../../src/adapters/classes/peer-library.adapter';

describe('PeerLibraryAdapter', () => {
	describe('toSummaries', () => {
		it('transforms libraries to summaries with file counts', () => {
			const libraries = [
				{ id: 'lib1', name: 'Movies', libraryType: 'movies' },
				{ id: 'lib2', name: 'Music', libraryType: 'music' }
			];
			const files = {
				lib1: [{ id: 'f1' }, { id: 'f2' }],
				lib2: [{ id: 'f3' }]
			};
			const result = peerLibraryAdapter.toSummaries(libraries as any, files as any);
			expect(result).toHaveLength(2);
			expect(result[0].fileCount).toBe(2);
			expect(result[1].fileCount).toBe(1);
		});

		it('handles missing files for a library', () => {
			const libraries = [{ id: 'lib1', name: 'Movies', libraryType: 'movies' }];
			const result = peerLibraryAdapter.toSummaries(libraries as any, {});
			expect(result[0].fileCount).toBe(0);
		});
	});

	describe('toFileInfos', () => {
		it('transforms library files to file infos', () => {
			const files = [{ id: 'f1', name: 'movie.mp4', extension: 'mp4', mediaType: 'video' }];
			const result = peerLibraryAdapter.toFileInfos(files as any);
			expect(result).toHaveLength(1);
			expect(result[0].name).toBe('movie.mp4');
			expect(result[0].extension).toBe('mp4');
		});
	});

	describe('libraryTypeLabel', () => {
		it('maps known types to labels', () => {
			expect(peerLibraryAdapter.libraryTypeLabel('movies' as any)).toBe('Movies');
			expect(peerLibraryAdapter.libraryTypeLabel('tv' as any)).toBe('TV Shows');
		});

		it('returns raw type for unknown', () => {
			expect(peerLibraryAdapter.libraryTypeLabel('music' as any)).toBe('music');
		});
	});
});
