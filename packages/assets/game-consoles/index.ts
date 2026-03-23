/// <reference path="../types.d.ts" />

import GBA from './GBA.png';
import GB from './GB.png';
import GBC from './GBC_1.png';
import SNES from './SNES.png';
import NES from './NES.png';
import MD_GEN from './MD_GEN.png';
import SMS from './SMS.png';
import N64 from './N64.png';
import PS1 from './PS1.png';
import PS2 from './PS2.png';
import NDS from './NDS.png';
import PCE from './PCE.png';
import GC from './GC.png';
import NGPC from './NGPC.png';
import LYNX from './LYNX.png';

export const CONSOLE_IMAGES: Record<number, string> = {
	5: GBA,
	4: GB,
	6: GBC,
	3: SNES,
	7: NES,
	1: MD_GEN,
	11: SMS,
	2: N64,
	12: PS1,
	21: PS2,
	18: NDS,
	8: PCE,
	47: GC,
	14: NGPC,
	13: LYNX
};
