import type { Transport } from './transport.type';
import { HttpTransport } from './http-transport';

let _transport: Transport = new HttpTransport();

export function setTransport(transport: Transport): void {
	_transport = transport;
}

export function getTransport(): Transport {
	return _transport;
}
