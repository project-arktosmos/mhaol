/**
 * Cloud-side firkin types. Re-exported from the shared `cloud-ui`
 * workspace package so the cloud and the browser-side player render off
 * the exact same shapes. Local aliases (`CloudFirkin`, `FirkinFile`,
 * `FirkinImage`) are preserved so existing imports keep working.
 */
import type {
	Firkin as SharedFirkin,
	FirkinArtist as SharedFirkinArtist,
	FirkinFile as SharedFirkinFile,
	FirkinFileType as SharedFirkinFileType,
	FirkinImage as SharedFirkinImage,
	FirkinTrailer as SharedFirkinTrailer
} from 'cloud-ui';

export type FirkinArtist = SharedFirkinArtist;
export type FirkinImage = SharedFirkinImage;
export type FirkinFile = SharedFirkinFile;
export type FirkinFileType = SharedFirkinFileType;
export type FirkinTrailer = SharedFirkinTrailer;
export type CloudFirkin = SharedFirkin;
