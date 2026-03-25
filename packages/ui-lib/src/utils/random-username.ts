const ADJECTIVES = [
	'Swift',
	'Brave',
	'Crimson',
	'Golden',
	'Silent',
	'Amber',
	'Copper',
	'Frozen',
	'Iron',
	'Lunar',
	'Misty',
	'Noble',
	'Rapid',
	'Shadow',
	'Solar',
	'Storm',
	'Velvet',
	'Wild',
	'Crystal',
	'Dusk',
	'Ember',
	'Frost',
	'Jade',
	'Onyx',
	'Rust',
	'Sage',
	'Silver',
	'Thorn',
	'Vivid',
	'Zinc'
];

const NOUNS = [
	'Falcon',
	'Panda',
	'Otter',
	'Raven',
	'Wolf',
	'Fox',
	'Lynx',
	'Hawk',
	'Bear',
	'Crane',
	'Viper',
	'Heron',
	'Tiger',
	'Eagle',
	'Cobra',
	'Bison',
	'Owl',
	'Shark',
	'Moose',
	'Jackal',
	'Mantis',
	'Osprey',
	'Panther',
	'Badger',
	'Condor',
	'Ferret',
	'Iguana',
	'Koala',
	'Newt',
	'Wren'
];

export function generateRandomUsername(): string {
	const adj = ADJECTIVES[Math.floor(Math.random() * ADJECTIVES.length)];
	const noun = NOUNS[Math.floor(Math.random() * NOUNS.length)];
	const num = Math.floor(Math.random() * 100);
	return `${adj}${noun}${num}`;
}
