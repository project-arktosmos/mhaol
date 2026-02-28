import type { PluginCompanion, PluginContext } from '../types';

export const signalingCompanion: PluginCompanion = {
	locals: {
		signalingPartyUrl: (ctx: PluginContext) => {
			return ctx.settingsRepo.get('signaling.partyUrl') ?? '';
		}
	}
};
