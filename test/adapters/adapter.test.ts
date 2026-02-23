import { describe, it, expect } from 'vitest';
import { AdapterClass } from '../../src/adapters/classes/adapter.class';

describe('AdapterClass', () => {
	describe('Initialization', () => {
		it('should initialize with correct id format', () => {
			const adapter = new AdapterClass('test-adapter');

			expect(adapter.id).toBe('adapter:test-adapter');
		});

		it('should accept different name formats', () => {
			const simpleAdapter = new AdapterClass('simple');
			const kebabAdapter = new AdapterClass('kebab-case-name');
			const snakeAdapter = new AdapterClass('snake_case_name');
			const camelAdapter = new AdapterClass('camelCaseName');

			expect(simpleAdapter.id).toBe('adapter:simple');
			expect(kebabAdapter.id).toBe('adapter:kebab-case-name');
			expect(snakeAdapter.id).toBe('adapter:snake_case_name');
			expect(camelAdapter.id).toBe('adapter:camelCaseName');
		});

		it('should handle empty string name', () => {
			const adapter = new AdapterClass('');

			expect(adapter.id).toBe('adapter:');
		});

		it('should handle names with special characters', () => {
			const adapter = new AdapterClass('test@adapter#123');

			expect(adapter.id).toBe('adapter:test@adapter#123');
		});

		it('should handle names with spaces', () => {
			const adapter = new AdapterClass('test adapter with spaces');

			expect(adapter.id).toBe('adapter:test adapter with spaces');
		});

		it('should create unique instances', () => {
			const adapter1 = new AdapterClass('adapter1');
			const adapter2 = new AdapterClass('adapter2');

			expect(adapter1.id).not.toBe(adapter2.id);
			expect(adapter1).not.toBe(adapter2);
		});

		it('should create identical ids for same name', () => {
			const adapter1 = new AdapterClass('same-name');
			const adapter2 = new AdapterClass('same-name');

			expect(adapter1.id).toBe(adapter2.id);
		});
	});

	describe('ID property', () => {
		it('should be a string', () => {
			const adapter = new AdapterClass('test');

			expect(typeof adapter.id).toBe('string');
		});

		it('should be readable', () => {
			const adapter = new AdapterClass('test');
			const id = adapter.id;

			expect(id).toBe('adapter:test');
		});

		it('should be mutable', () => {
			const adapter = new AdapterClass('test');
			adapter.id = 'adapter:modified';

			expect(adapter.id).toBe('adapter:modified');
		});
	});

	describe('Multiple instances', () => {
		it('should allow creating multiple adapters', () => {
			const adapters = [
				new AdapterClass('adapter1'),
				new AdapterClass('adapter2'),
				new AdapterClass('adapter3')
			];

			expect(adapters).toHaveLength(3);
			expect(adapters[0].id).toBe('adapter:adapter1');
			expect(adapters[1].id).toBe('adapter:adapter2');
			expect(adapters[2].id).toBe('adapter:adapter3');
		});

		it('should maintain separate state for each instance', () => {
			const adapter1 = new AdapterClass('first');
			const adapter2 = new AdapterClass('second');

			adapter1.id = 'modified:first';

			expect(adapter1.id).toBe('modified:first');
			expect(adapter2.id).toBe('adapter:second');
		});
	});

	describe('Extensibility', () => {
		it('should be extendable', () => {
			class ExtendedAdapter extends AdapterClass {
				customProperty: string;

				constructor(name: string, customProperty: string) {
					super(name);
					this.customProperty = customProperty;
				}

				customMethod(): string {
					return `${this.id} - ${this.customProperty}`;
				}
			}

			const extended = new ExtendedAdapter('extended', 'custom');

			expect(extended.id).toBe('adapter:extended');
			expect(extended.customProperty).toBe('custom');
			expect(extended.customMethod()).toBe('adapter:extended - custom');
		});

		it('should support method overriding', () => {
			class OverriddenAdapter extends AdapterClass {
				constructor(name: string) {
					super(name);
					// Override the id with custom logic
					this.id = `custom:${name}:override`;
				}
			}

			const overridden = new OverriddenAdapter('test');

			expect(overridden.id).toBe('custom:test:override');
		});
	});

	describe('Type checking', () => {
		it('should be an instance of AdapterClass', () => {
			const adapter = new AdapterClass('test');

			expect(adapter).toBeInstanceOf(AdapterClass);
		});

		it('should have constructor property', () => {
			const adapter = new AdapterClass('test');

			expect(adapter.constructor).toBe(AdapterClass);
		});
	});

	describe('Edge cases', () => {
		it('should handle numeric string names', () => {
			const adapter = new AdapterClass('12345');

			expect(adapter.id).toBe('adapter:12345');
		});

		it('should handle very long names', () => {
			const longName = 'a'.repeat(1000);
			const adapter = new AdapterClass(longName);

			expect(adapter.id).toBe(`adapter:${longName}`);
		});

		it('should handle unicode characters', () => {
			const adapter = new AdapterClass('测试适配器');

			expect(adapter.id).toBe('adapter:测试适配器');
		});

		it('should handle emoji in names', () => {
			const adapter = new AdapterClass('adapter🚀test');

			expect(adapter.id).toBe('adapter:adapter🚀test');
		});
	});
});
