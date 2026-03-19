import { readFileSync, writeFileSync } from 'fs';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const enPath = join(__dirname, '../src/lib/i18n/locales/en.json');
const qqPath = join(__dirname, '../src/lib/i18n/locales/qq.json');

/**
 * Recursively replace all string values with 'QQQQQ'
 */
function replaceWithQQ(obj) {
	if (typeof obj === 'string') {
		return 'QQQQQ';
	}

	if (Array.isArray(obj)) {
		return obj.map(replaceWithQQ);
	}

	if (obj !== null && typeof obj === 'object') {
		const result = {};
		for (const [key, value] of Object.entries(obj)) {
			result[key] = replaceWithQQ(value);
		}
		return result;
	}

	return obj;
}

try {
	// Read the English translation file
	const enContent = readFileSync(enPath, 'utf-8');
	const enData = JSON.parse(enContent);

	// Replace all values with 'QQQQQ'
	const qqData = replaceWithQQ(enData);

	// Write the QQ translation file
	writeFileSync(qqPath, JSON.stringify(qqData, null, '\t'));

	console.log('✓ Generated qq.json successfully');
} catch (error) {
	console.error('Error generating qq.json:', error);
	process.exit(1);
}
