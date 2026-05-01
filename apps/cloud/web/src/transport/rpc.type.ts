import type { TransportMethod } from './transport.type';

export interface RpcRequest {
	id: string;
	type: 'request';
	method: TransportMethod;
	path: string;
	headers?: Record<string, string>;
	body?: string;
}

export interface RpcResponse {
	id: string;
	type: 'response';
	status: number;
	statusText: string;
	headers?: Record<string, string>;
	body?: string;
	chunked?: boolean;
	totalChunks?: number;
}

export interface RpcChunk {
	id: string;
	type: 'chunk';
	seq: number;
	data: string;
	final: boolean;
}

export interface RpcSubscribe {
	id: string;
	type: 'subscribe';
	path: string;
}

export interface RpcUnsubscribe {
	id: string;
	type: 'unsubscribe';
}

export interface RpcStreamEvent {
	id: string;
	type: 'stream-event';
	eventType: string;
	data: string;
}

export interface RpcStreamEnd {
	id: string;
	type: 'stream-end';
}

export type RpcMessage =
	| RpcRequest
	| RpcResponse
	| RpcChunk
	| RpcSubscribe
	| RpcUnsubscribe
	| RpcStreamEvent
	| RpcStreamEnd;

export interface RpcEnvelope {
	channel: 'rpc';
	payload: RpcMessage;
}
