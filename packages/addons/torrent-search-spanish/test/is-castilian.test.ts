import { describe, it, expect } from 'vitest';
import { isCastilianRelease } from '../src/is-castilian';

describe('isCastilianRelease', () => {
	it('detects castilian markers', () => {
		expect(isCastilianRelease('Barbie.2023.1080p.BluRay.x264.SPANISH')).toBe(true);
		expect(isCastilianRelease('Barbie (2023) [Castellano]')).toBe(true);
		expect(isCastilianRelease('Barbie.2023.1080p.WEB-DL.DUAL.CAST')).toBe(true);
		expect(isCastilianRelease('Barbie.2023.[ESP][1080p]')).toBe(true);
		expect(isCastilianRelease('Barbie 2023 español')).toBe(true);
		expect(isCastilianRelease('Barbie.2023.es-ES.1080p.WEBRip')).toBe(true);
	});

	it('rejects latin american releases', () => {
		expect(isCastilianRelease('Barbie.2023.LATAM.1080p')).toBe(false);
		expect(isCastilianRelease('Barbie.2023.Latino.1080p')).toBe(false);
		expect(isCastilianRelease('Barbie.2023.es-MX.1080p.WEBRip')).toBe(false);
		expect(isCastilianRelease('Barbie 2023 español latino')).toBe(false);
		// The user's reported case
		expect(
			isCastilianRelease('Toy.Story.2.1999.1080p.BluRay.ENG.LATINO.DD5.1.H264-BEN.THE.MEN')
		).toBe(false);
	});

	it('rejects english releases', () => {
		expect(isCastilianRelease('Barbie.2023.1080p.BluRay.x264.ENGLISH')).toBe(false);
		expect(isCastilianRelease('Barbie 2023 1080p WEB-DL')).toBe(false);
	});

	it('does not match substrings', () => {
		expect(isCastilianRelease('Spawn.1997.1080p')).toBe(false);
		expect(isCastilianRelease('Castle.S01.1080p')).toBe(false);
		expect(isCastilianRelease('Broadcast.News.1987.1080p')).toBe(false);
		expect(isCastilianRelease('Espionage.2020.1080p')).toBe(false);
		expect(isCastilianRelease('Late.Night.2019.1080p')).toBe(false);
	});
});
