import { describe, it, expect } from 'vitest';
import { parseStatusLine } from './status-parser';

describe('parseStatusLine', () => {
	it('extracts claude model name', () => {
		const result = parseStatusLine('Using claude-sonnet-4-20250514');
		expect(result.model).toBe('claude-sonnet-4-20250514');
	});

	it('extracts gpt model name', () => {
		const result = parseStatusLine('Model: gpt-4.1-turbo');
		expect(result.model).toBe('gpt-4.1-turbo');
	});

	it('extracts gemini model name', () => {
		const result = parseStatusLine('gemini-2.5');
		expect(result.model).toBe('gemini-2.5');
	});

	it('extracts reasoning effort', () => {
		const result = parseStatusLine('reasoning_effort: high');
		expect(result.reasoning).toBe('high');
	});

	it('extracts Extra High reasoning', () => {
		const result = parseStatusLine('Extra High reasoning');
		expect(result.reasoning).toBe('extra-high');
	});

	it('extracts input tokens', () => {
		const result = parseStatusLine('1,234 input tokens');
		expect(result.inputTokens).toBe(1234);
	});

	it('extracts output tokens', () => {
		const result = parseStatusLine('5678 output tokens');
		expect(result.outputTokens).toBe(5678);
	});

	it('extracts context percent', () => {
		const result = parseStatusLine('context: 42%');
		expect(result.contextPercent).toBe(42);
	});

	it('extracts context left percent', () => {
		const result = parseStatusLine('75% context left');
		expect(result.contextPercent).toBe(25);
	});

	it('extracts absolute context usage with k suffix', () => {
		const result = parseStatusLine('50k / 200k');
		expect(result.contextUsed).toBe(50000);
		expect(result.contextTotal).toBe(200000);
	});

	it('extracts absolute context usage without k on used', () => {
		const result = parseStatusLine('500 / 200k');
		expect(result.contextUsed).toBe(500);
		expect(result.contextTotal).toBe(200000);
	});

	it('returns empty for unrelated lines', () => {
		const result = parseStatusLine('hello world');
		expect(result).toEqual({});
	});

	it('extracts multiple fields from one line', () => {
		const result = parseStatusLine('claude-opus-4-20250514 1,000 input tokens ctx 80%');
		expect(result.model).toBe('claude-opus-4-20250514');
		expect(result.inputTokens).toBe(1000);
		expect(result.contextPercent).toBe(80);
	});
});
