# Website

**Location:** `apps/website/`
**Framework:** SvelteKit (Svelte 5 runes) + Vite + Tailwind 4 + DaisyUI 5
**pnpm package:** `mhaol-website`

The public informational site for the Mhaol project. Pure static SvelteKit SPA — no backend, no auth, no API. Builds to `apps/website/dist-static/` via `@sveltejs/adapter-static` and is meant to be deployed to any static host (GitHub Pages, Netlify, an S3 bucket, the embedded `mhaol-backend` if you want, etc.).

It shares `cloud-ui` for icons and display components so the website's visual language matches the in-app experience.

## Source structure

```
apps/website/
├── package.json           # pnpm package "mhaol-website"
├── svelte.config.js       # path aliases ($components, $utils, $types, $data)
├── vite.config.ts         # port 9897 (avoids the cloud's 9898/9899/9900/9901)
├── tsconfig.json
├── eslint.config.js
└── src/
    ├── routes/
    │   ├── +layout.svelte    # header / footer chrome
    │   ├── +layout.ts        # prerender = true, ssr = false
    │   └── +page.svelte      # composes the landing-page sections
    ├── components/
    │   ├── core/             # ThemeToggle, etc.
    │   └── landing/          # Hero, Features, MediaTypes, Apps, HowItWorks, Install
    └── css/
        ├── app.css           # tailwind + daisyui entry
        └── themes.css        # one light theme + one dark theme
```

## Aliases

```js
alias: {
  $components: 'src/components',
  $utils: 'src/utils',
  $types: 'src/types',
  $data: 'src/data'
}
```

Plus the SvelteKit-reserved `$lib` and `$app/*`.

## Theming

Two daisyUI themes (`light` + `dark`). The `ThemeToggle` component persists the choice to `localStorage["mhaol-website:theme"]` and a small inline `<script>` in `app.html` re-applies it before first paint to avoid a flash.

## Icons

Same rules as the rest of the monorepo: use `<Icon name="<author>/<name>" />` from `cloud-ui` for every glyph, never emoji. See the root `CLAUDE.md` "Icons" section.

## Running

```bash
# Dev — hot-reload on http://localhost:9897
pnpm dev:website

# Production build → apps/website/dist-static/
pnpm build:website
```
