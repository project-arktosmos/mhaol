/// <reference types="@sveltejs/kit" />

declare namespace App {
	// interface Locals {}
	// interface PageData {}
	// interface Platform {}
}

interface ImportMetaEnv {
	readonly VITE_MHAOL_HEALTH_APPS?: string;
}

interface ImportMeta {
	readonly env: ImportMetaEnv;
}
