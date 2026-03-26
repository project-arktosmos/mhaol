# WASM Browser Emulators for Retro Consoles

Research document mapping each console on the `/videogames` page to viable WebAssembly emulators that can run in the browser.

---

## Summary

| Console | RA ID | WASM Status | Best Candidate |
|---------|-------|-------------|----------------|
| Game Boy Advance | 5 | YES | gbajs3 (mGBA WASM core) |
| Game Boy | 4 | YES | WasmBoy (AssemblyScript) |
| Game Boy Color | 6 | YES | WasmBoy (AssemblyScript) |
| SNES/Super Famicom | 3 | YES | byuu-web (bsnes port) |
| NES/Famicom | 7 | YES | nes-rust (Rust+WASM) |
| Genesis/Mega Drive | 1 | YES | wasm-genplus (Genesis-Plus-GX) |
| Master System | 11 | YES | jgenesis (Rust, multi-system) |
| Nintendo 64 | 2 | YES | N64Wasm (ParaLLEl Core) |
| PlayStation | 12 | YES | PCSX-wasm (PCSX-R port) |
| PlayStation 2 | 21 | EXPERIMENTAL | Play! (experimental WASM) |
| Nintendo DS | 18 | YES | ds-anywhere (melonDS WASM) |
| Atari 2600 | 9 | YES | atari2600-wasm (AssemblyScript) |
| PC Engine/TurboGrafx-16 | 8 | YES | webrcade (Mednafen PCE core) |
| Atari 7800 | 10 | YES | webrcade (ProSystem core) |
| Nintendo GameCube | 47 | NO | None available |
| Neo Geo Pocket | 14 | YES | webrcade (Beetle NeoPop core) |
| Atari Lynx | 13 | YES | webrcade (Handy core) |
| Atari Jaguar | 17 | YES | EmulatorJS (Virtual Jaguar core) |

**Totals: 16 YES, 1 EXPERIMENTAL (PS2), 1 NO (GameCube)**

---

## Per-Console Details

### Game Boy Advance (ID 5) - YES

**Best candidate:** [gbajs3](https://github.com/thenick775/gbajs3)
- Uses mGBA WASM core via Emscripten
- Actively maintained, ~80 stars
- Full-featured: save states, fast forward, gamepad support
- Designed as an all-in-one web app (React frontend + optional backend)

**Alternatives:**
- [web-gba-emu](https://github.com/nealmm/web-gba-emu) - C++ to JS/WASM port, IndexedDB saves, works on mobile
- [wasm-gba](https://github.com/felixzhuologist/wasm-gba) - Standalone browser GBA emulator
- [mGBA WASM fork](https://github.com/thenick775/mgba) - The underlying mGBA fork with WASM bindings

**Multi-system fallback:** Emulatrix (vba_next core), EmulatorJS

---

### Game Boy (ID 4) - YES

**Best candidate:** [WasmBoy](https://github.com/torch2424/wasmboy)
- Written in AssemblyScript, compiled to WASM
- ~1.5k stars, well-documented
- Supports Game Boy AND Game Boy Color
- Demos built with Preact and Svelte
- Includes debugger for homebrew development

**Alternatives:**
- [wasm-gb](https://github.com/andrewimm/wasm-gb) - GB emulator in WASM + WebGL 2.0

**Multi-system fallback:** Emulatrix (gambatte core), EmulatorJS

---

### Game Boy Color (ID 6) - YES

**Best candidate:** [WasmBoy](https://github.com/torch2424/wasmboy)
- Same as Game Boy above, supports both GB and GBC
- Single emulator covers both systems

**Multi-system fallback:** Emulatrix (gambatte core), EmulatorJS

---

### SNES/Super Famicom (ID 3) - YES

**Best candidate:** [byuu-web](https://github.com/Wizcorp/byuu-web)
- Web port of bsnes/higan (high-accuracy SNES emulator)
- Combines accuracy of higan with simplicity of bsnes
- Live demo available at https://wizcorp.github.io/byuu-web

**Alternatives:**
- [jgenesis](https://github.com/jsgroth/jgenesis) - Rust multi-system emulator with WASM web frontend, supports SNES

**Multi-system fallback:** Emulatrix (snes9x2010 core), EmulatorJS

---

### NES/Famicom (ID 7) - YES

**Best candidate:** [nes-rust](https://github.com/takahirox/nes-rust)
- Written in Rust, compiles to WASM
- Clean architecture, good performance

**Alternatives:**
- [rustynes](https://github.com/bokuweb/rustynes) - NES emulator in Rust+WASM
- [rusticnes-wasm](https://github.com/zeta0134/rusticnes-wasm) - Web interface for RusticNES via wasm-bindgen
- [SaltyNES](https://github.com/workhorsy/SaltyNES) - NES in WASM, has live demo
- [wasm-nes (kabukki)](https://github.com/kabukki/wasm-nes) - Rust, compiled via wasm-pack
- [wasm-nes (irdcat)](https://github.com/irdcat/wasm-nes) - C++, compiled via Emscripten

**Multi-system fallback:** Emulatrix (fceumm core), EmulatorJS

---

### Genesis/Mega Drive (ID 1) - YES

**Best candidate:** [wasm-genplus](https://github.com/h1romas4/wasm-genplus)
- WebAssembly port of Genesis-Plus-GX (well-known accurate Genesis emulator)
- Dedicated Genesis WASM project

**Alternatives:**
- [jgenesis](https://github.com/jsgroth/jgenesis) - Rust multi-system with WASM, supports Genesis + Sega CD

**Multi-system fallback:** Emulatrix (genesis_plus_gx core), EmulatorJS

---

### Master System (ID 11) - YES

**Best candidate:** [jgenesis](https://github.com/jsgroth/jgenesis)
- Rust multi-system emulator with WASM web frontend
- Supports Master System + Game Gear natively
- Active development

**Multi-system fallback:** EmulatorJS

---

### Nintendo 64 (ID 2) - YES

**Best candidate:** [N64Wasm](https://github.com/nbarkhina/N64Wasm)
- ~646 stars, port of RetroArch ParaLLEl Core to WASM
- Good 3D game compatibility at full speed on mid-range hardware
- Works on iPhone 13 and Xbox Series X browser
- Active community

**Multi-system fallback:** EmulatorJS, webrcade

---

### PlayStation (ID 12) - YES

**Best candidate:** [PCSX-wasm](https://github.com/kxkx5150/PCSX-wasm)
- Modified PCSX-R compiled with Emscripten to WASM
- Dedicated PS1 WASM project

**Alternatives:**
- [wasmpsx](https://github.com/js-emulators/wasmpsx) - Embeddable PS1 emulator fork
- [lrusso/PlayStation](https://github.com/lrusso/PlayStation) - PS1 in JS+WASM, has live demo

**Multi-system fallback:** EmulatorJS, webrcade

---

### PlayStation 2 (ID 21) - EXPERIMENTAL

**Best candidate:** [Play!](https://github.com/jpd002/Play-)
- PS2 emulator with experimental WASM build
- Live demo at playjs.purei.org
- Uses built-in HLE BIOS (no external BIOS needed)
- Performance is limited in browser — PS2 is very demanding
- Not all games run well

**Notes:** PS2 emulation in the browser is at the bleeding edge. PCSX2 (the most accurate PS2 emulator) has no WASM port. Play! is the only option and it's experimental.

**Multi-system fallback:** None — Play! is the only option

---

### Nintendo DS (ID 18) - YES

**Best candidate:** [ds-anywhere](https://github.com/brxxn/ds-anywhere)
- Fork of melonDS compiled to WASM via Emscripten
- TypeScript Preact/Vite frontend
- Designed for secure in-browser emulation

**Alternatives:**
- [desmume-wasm](https://github.com/44670/desmume-wasm) - DeSmuME WASM port, designed for iPhone/iPad
- [desmond](https://github.com/js-emulators/desmond) - Embeddable DeSmuME-wasm bundle

**Multi-system fallback:** EmulatorJS, webrcade

---

### Atari 2600 (ID 9) - YES

**Best candidate:** [atari2600-wasm](https://github.com/ColinEberhardt/atari2600-wasm)
- Written in AssemblyScript, compiled to WASM
- Dedicated Atari 2600 project

**Alternatives:**
- [Atari2600.wasm](https://github.com/chun-baoluo/Atari2600.wasm) - C++ WASM experiment

**Multi-system fallback:** EmulatorJS

---

### PC Engine/TurboGrafx-16 (ID 8) - YES

**Best candidate:** [webrcade-app-mednafen](https://github.com/webrcade/webrcade-app-mednafen)
- Mednafen PCE core compiled to WASM
- Part of webrcade platform with save state support

**Alternatives:**
- [jspce](https://github.com/abnuo/jspce) - JavaScript PC Engine emulator (not WASM, but runs in browser)

**Multi-system fallback:** EmulatorJS

---

### Atari 7800 (ID 10) - YES

**Best candidate:** [webrcade](https://github.com/webrcade/webrcade)
- ProSystem core compiled to WASM
- Save state support

**Multi-system fallback:** EmulatorJS (Atari 7800 core)

---

### Nintendo GameCube (ID 47) - NO

**No functional browser WASM emulator exists.**

- Dolphin (the standard GC/Wii emulator) has no working WASM port
- [dolphin-web](https://github.com/aiden1000oo/dolphin-web) exists but is not a functional WASM build
- GameCube emulation requires JIT recompilation, GPU acceleration, and memory resources beyond what current browser WASM runtimes can provide
- This may change in the future as WASM evolves (WASM threads, SIMD, etc.)

---

### Neo Geo Pocket (ID 14) - YES

**Best candidate:** [webrcade-app-mednafen](https://github.com/webrcade/webrcade-app-mednafen)
- Beetle NeoPop core compiled to WASM
- Covers Neo Geo Pocket and Neo Geo Pocket Color

**Multi-system fallback:** EmulatorJS

---

### Atari Lynx (ID 13) - YES

**Best candidate:** [webrcade-app-mednafen](https://github.com/webrcade/webrcade-app-mednafen)
- Handy core compiled to WASM
- Part of webrcade platform

**Multi-system fallback:** EmulatorJS

---

### Atari Jaguar (ID 17) - YES

**Best candidate:** [EmulatorJS](https://github.com/EmulatorJS/EmulatorJS)
- Virtual Jaguar core compiled to WASM
- Note: Virtual Jaguar has limited game compatibility compared to other emulator cores
- No dedicated standalone Atari Jaguar WASM emulator exists

**Multi-system fallback:** EmulatorJS is the primary (and only) option

---

## Multi-System Platforms

These platforms bundle multiple emulator cores into a single WASM-based web frontend:

### [EmulatorJS](https://github.com/EmulatorJS/EmulatorJS)
- Web frontend for RetroArch compiled to Emscripten/WASM
- Covers nearly all systems listed above
- Public CDN available at cdn.emulatorjs.org
- Library/plugin design, embeddable in any site

### [webrcade](https://github.com/webrcade/webrcade)
- Feed-driven gaming platform, WASM-based
- Covers: NES, SNES, Genesis, N64, PS1, NDS, GBA, GB/GBC, Master System, Atari 2600/7800, Atari Lynx, Neo Geo Pocket, PC Engine, and more
- Save state support, works on Xbox/iOS/Android/desktop browsers

### [Emulatrix](https://github.com/lrusso/Emulatrix)
- JS+WASM emulator with mobile compatibility
- Covers: NES, SNES, Genesis, GBA, GB, GBC, MAME, DOSBox
- Uses RetroArch cores (fceumm, snes9x2010, gambatte, vba_next, genesis_plus_gx)

### [jgenesis](https://github.com/jsgroth/jgenesis)
- Rust multi-system emulator with WASM web frontend
- Covers: Genesis, Sega CD, SNES, Master System, Game Gear
- High accuracy, active development
