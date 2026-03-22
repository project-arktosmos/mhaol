import { apiUrl } from "frontend/lib/api-base";
import type { PageLoad } from "./$types";

export const load: PageLoad = async ({ fetch }) => {
  try {
    const res = await fetch(apiUrl("/api/media"));
    if (!res.ok) {
      const text = await res.text().catch(() => "");
      return { error: `Backend ${res.status}: ${text || res.statusText}` };
    }
    return await res.json();
  } catch (err) {
    return {
      error: `Backend unreachable: ${err instanceof Error ? err.message : String(err)}`,
    };
  }
};
