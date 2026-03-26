import type { Transport } from './transport.type';
import { HttpTransport } from './http-transport';

let _transport: Transport = new HttpTransport();
let _ready = false;

export function setTransport(transport: Transport): void {
	_transport = transport;
	_ready = true;
}

export function getTransport(): Transport {
	return _transport;
}

export function isTransportReady(): boolean {
	return _ready;
}
