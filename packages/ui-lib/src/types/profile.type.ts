export interface UserProfile {
	username: string;
	wallet: string;
}

export interface RemoteProfile {
	username: string;
	wallet: string;
	added_at: string;
	favoriteCount: number;
}

export interface ProfileDetail {
	profile: RemoteProfile;
	favorites: {
		id: string;
		wallet: string;
		service: string;
		service_id: string;
		label: string;
		created_at: string;
	}[];
}

export interface ProfileState {
	loading: boolean;
	local: UserProfile;
	remoteProfiles: RemoteProfile[];
	error: string | null;
}
