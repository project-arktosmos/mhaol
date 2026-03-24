<script lang="ts">
	import Hero from 'ui-lib/components/landing/Hero.svelte';
	import Features from 'ui-lib/components/landing/Features.svelte';
	import SmartSearch from 'ui-lib/components/landing/SmartSearch.svelte';
	import Addons from 'ui-lib/components/landing/Addons.svelte';
	import Footer from 'ui-lib/components/landing/Footer.svelte';
	import LandingNavbar from 'ui-lib/components/landing/LandingNavbar.svelte';
	import { themeService } from 'ui-lib/services/theme.service';

	themeService.initialize();
</script>

<svelte:head>
	<title>Mhaol Media — Open-Source Media Server</title>
</svelte:head>

<LandingNavbar navItems={[{ href: '/connect', label: 'Connect' }]} />
<Hero />
<Features />
<SmartSearch />
<Addons />
<Footer />
