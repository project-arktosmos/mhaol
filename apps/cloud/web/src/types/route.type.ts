export interface RouteEntry {
	/** The URL path, e.g. "/movies" or "/movies/[id]" */
	path: string;
	/** Human-readable display name, e.g. "Movies" */
	label: string;
	/** Whether this route segment contains a dynamic parameter like [id] */
	isDynamic: boolean;
	/** Child routes nested under this path */
	children: RouteEntry[];
}
