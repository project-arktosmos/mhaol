<script lang="ts">
	import { Icon, type IconName } from 'cloud-ui';

	type Platform = {
		icon: IconName;
		title: string;
		intro: string;
		steps: string[];
		callout?: string;
	};

	const platforms: Platform[] = [
		{
			icon: 'delapouite/apple-core',
			title: 'macOS',
			intro:
				'Gatekeeper blocks the first launch with “Apple could not verify Mhaol Cloud is free of malware”. Two ways past it:',
			steps: [
				'Drag Mhaol Cloud.app from the .dmg into /Applications.',
				'In Finder, right-click (or Control-click) the app and choose Open.',
				'Confirm Open in the dialog. macOS remembers the choice for future launches.',
				'Alternative: open System Settings → Privacy & Security, scroll to “Mhaol Cloud was blocked…”, and click Open Anyway.'
			],
			callout:
				'If you prefer the terminal: xattr -dr com.apple.quarantine "/Applications/Mhaol Cloud.app"'
		},
		{
			icon: 'delapouite/computer-fan',
			title: 'Windows',
			intro:
				'SmartScreen pops up “Windows protected your PC” because the installer isn’t Authenticode-signed. To allow it through:',
			steps: [
				'Right-click the downloaded installer → Properties.',
				'At the bottom of the General tab, tick Unblock and click OK.',
				'Double-click the installer. SmartScreen may still show — click More info → Run anyway.',
				'If your AV quarantines the file, restore it from the AV’s history and re-run.'
			],
			callout:
				'On Windows 11 you can also click More info → Run anyway directly from the SmartScreen dialog without the Properties step.'
		},
		{
			icon: 'lorc/android-mask',
			title: 'Android (mobile + TV)',
			intro:
				'Sideloading an unsigned APK requires two grants: per-source “install unknown apps” permission, and a Play Protect bypass.',
			steps: [
				'Transfer the .apk to the device (USB, the device’s browser, or a file-manager app).',
				'Open it from the source app. Android prompts: “For your security, your device isn’t allowed to install unknown apps from this source.” Tap Settings.',
				'Toggle Allow from this source on for that source app, then back out and re-open the .apk.',
				'Play Protect shows “Blocked by Play Protect” → tap More details → Install anyway.',
				'Android TV: Settings → Device Preferences → Security & restrictions → Unknown sources → enable for the source app, then sideload.'
			]
		}
	];
</script>

<section id="install" class="border-b border-base-300 bg-base-100">
	<div class="mx-auto w-full max-w-6xl px-4 py-20">
		<div class="mb-8 max-w-3xl">
			<h2 class="text-3xl font-bold tracking-tight md:text-4xl">Install</h2>
			<p class="mt-3 text-base-content/70">
				Every release builds the desktop, headless, and Android shells in CI and attaches them as
				artifacts to the GitHub release. Grab the right artifact for your device — they all run the
				same backend and the same SPA.
			</p>
		</div>

		<div class="mb-8 flex flex-wrap items-center gap-3">
			<a
				href="https://github.com/project-arktosmos/mhaol/releases"
				rel="noopener"
				class="btn gap-2 btn-primary"
			>
				<Icon name="delapouite/cloud-download" size={18} />
				Latest release on GitHub
			</a>
			<span class="inline-flex items-center gap-2 text-sm text-base-content/70">
				<Icon name="lorc/cog-lock" size={16} />
				Builds are unsigned today, so each OS asks for a one-time approval.
			</span>
		</div>

		<div class="grid gap-5 md:grid-cols-2 xl:grid-cols-3">
			{#each platforms as platform (platform.title)}
				<div class="flex flex-col rounded-box border border-base-300 bg-base-100 p-5">
					<div class="flex items-center gap-3">
						<div
							class="flex h-11 w-11 items-center justify-center rounded-md bg-primary/10 text-primary"
						>
							<Icon name={platform.icon} size={24} />
						</div>
						<div class="min-w-0 flex-1">
							<div class="font-semibold">{platform.title}</div>
						</div>
					</div>

					<p class="mt-4 text-sm text-base-content/80">{platform.intro}</p>

					<ol class="mt-3 list-decimal space-y-2 pl-5 text-sm text-base-content/80">
						{#each platform.steps as step (step)}
							<li>{step}</li>
						{/each}
					</ol>

					{#if platform.callout}
						<div
							class="mt-4 rounded-md border border-base-300 bg-base-200 p-3 text-xs text-base-content/70"
						>
							<div class="mb-1 inline-flex items-center gap-1.5 font-semibold text-base-content/90">
								<Icon name="delapouite/info" size={14} />
								Tip
							</div>
							{platform.callout}
						</div>
					{/if}
				</div>
			{/each}
		</div>

		<div class="mt-10 rounded-box border border-base-300 bg-base-200 p-5">
			<div class="flex items-start gap-3">
				<div class="text-primary">
					<Icon name="lorc/cog-lock" size={22} />
				</div>
				<div class="text-sm text-base-content/80">
					<div class="font-semibold text-base-content">Why the security prompts?</div>
					<p class="mt-1">
						Mhaol's CI doesn't yet have an Apple Developer ID, an Authenticode certificate, or a
						Play Store listing. Every release ships as a plain artifact, so the host OS treats it as
						untrusted on first launch. The hops above are the standard way to consent — they don't
						compromise your system, they just acknowledge that you trust this binary. Once you
						approve, subsequent launches and auto-updates run without prompts.
					</p>
				</div>
			</div>
		</div>
	</div>
</section>
