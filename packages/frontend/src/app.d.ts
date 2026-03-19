declare global {
	namespace App {
		// No server-side locals — frontend is static, all API calls go to the Rust backend
	}
}

export {};
