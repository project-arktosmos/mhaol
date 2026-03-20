/// <reference path="../../../packages/frontend/src/musicbrainz.d.ts" />
/// <reference path="../../../packages/frontend/src/youtube.d.ts" />
/// <reference path="../../../packages/frontend/src/lyrics.d.ts" />
/// <reference path="../../../packages/frontend/src/image-tagger.d.ts" />

declare global {
  namespace App {
    // No server-side locals — frontend is static, all API calls go to the Rust backend
  }
}

export {};
