import { describe, it, expect } from 'vitest';
import { encodePayload, decodePayload, fitsInQrCode } from '../../src/utils/p2p/sdp-codec';
import type { P2pSignalingPayload } from '../../src/types/p2p.type';

describe('SDP Codec', () => {
	const samplePayload: P2pSignalingPayload = {
		type: 'offer',
		sdp: 'v=0\r\no=- 1234 1 IN IP4 0.0.0.0\r\ns=-\r\nt=0 0\r\na=group:BUNDLE 0\r\nm=application 9 UDP/DTLS/SCTP webrtc-datachannel\r\nc=IN IP4 0.0.0.0\r\na=mid:0\r\n',
		address: '0x1234567890abcdef1234567890abcdef12345678'
	};

	it('should encode and decode a payload roundtrip', () => {
		const encoded = encodePayload(samplePayload);
		const decoded = decodePayload(encoded);
		expect(decoded).toEqual(samplePayload);
	});

	it('should produce a base64 string', () => {
		const encoded = encodePayload(samplePayload);
		expect(() => atob(encoded)).not.toThrow();
	});

	it('should handle answer type payloads', () => {
		const answerPayload: P2pSignalingPayload = {
			type: 'answer',
			sdp: 'v=0\r\no=- 5678 1 IN IP4 0.0.0.0\r\ns=-\r\n',
			address: '0xabcdef1234567890abcdef1234567890abcdef12'
		};
		const encoded = encodePayload(answerPayload);
		const decoded = decodePayload(encoded);
		expect(decoded).toEqual(answerPayload);
	});

	it('should return true for fitsInQrCode with small payloads', () => {
		const encoded = encodePayload(samplePayload);
		expect(fitsInQrCode(encoded)).toBe(true);
	});

	it('should return false for fitsInQrCode with oversized strings', () => {
		const oversized = 'x'.repeat(5000);
		expect(fitsInQrCode(oversized)).toBe(false);
	});

	it('should handle whitespace in encoded input', () => {
		const encoded = encodePayload(samplePayload);
		const withWhitespace = '  ' + encoded + '  ';
		const decoded = decodePayload(withWhitespace);
		expect(decoded).toEqual(samplePayload);
	});

	it('should throw on invalid base64 input', () => {
		expect(() => decodePayload('not-valid-data!!!')).toThrow();
	});

	it('should throw on corrupted compressed data', () => {
		const validBase64 = btoa('this is not compressed data');
		expect(() => decodePayload(validBase64)).toThrow();
	});
});
