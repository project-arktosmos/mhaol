export interface AuthProvider {
	getAuthHeaders(): Promise<Record<string, string>>;
	getAuthQueryParams(): Promise<string>;
}

export class PassportAuthProvider implements AuthProvider {
	private address: string;
	private signMessage: (msg: string) => Promise<string>;

	constructor(address: string, signMessage: (msg: string) => Promise<string>) {
		this.address = address;
		this.signMessage = signMessage;
	}

	async getAuthHeaders(): Promise<Record<string, string>> {
		const timestamp = String(Date.now());
		const message = `mhaol-auth:${timestamp}`;
		const signature = await this.signMessage(message);
		return {
			'x-auth-address': this.address,
			'x-auth-signature': signature,
			'x-auth-timestamp': timestamp
		};
	}

	async getAuthQueryParams(): Promise<string> {
		const headers = await this.getAuthHeaders();
		const params = new URLSearchParams();
		for (const [key, value] of Object.entries(headers)) {
			params.set(key, value);
		}
		return params.toString();
	}
}
