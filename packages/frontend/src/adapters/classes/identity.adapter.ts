import { AdapterClass } from 'frontend/adapters/classes/adapter.class';

export class IdentityAdapter extends AdapterClass {
	constructor() {
		super('identity');
	}

	shortAddress(address: string): string {
		if (!address.startsWith('0x') || address.length < 10) return address;
		return `${address.slice(0, 6)}...${address.slice(-4)}`;
	}
}

export const identityAdapter = new IdentityAdapter();
