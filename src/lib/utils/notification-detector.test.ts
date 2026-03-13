import { describe, it, expect } from 'vitest';
import { detectNotification, createLineBuffer } from './notification-detector';

describe('detectNotification', () => {
	it('matches claude prompt', () => {
		const result = detectNotification('some text \u276F ');
		expect(result.matched).toBe(true);
		expect(result.pattern).toBe('claude-prompt');
	});

	it('matches claude confirm', () => {
		const result = detectNotification('Continue? (y/n)');
		expect(result.matched).toBe(true);
		expect(result.pattern).toBe('claude-confirm');
	});

	it('matches claude waiting', () => {
		const result = detectNotification('Waiting for your response');
		expect(result.matched).toBe(true);
		expect(result.pattern).toBe('claude-waiting');
	});

	it('matches claude permission', () => {
		const result = detectNotification('Press (y) to approve');
		expect(result.matched).toBe(true);
		expect(result.pattern).toBe('claude-permission');
	});

	it('matches codex prompt', () => {
		const result = detectNotification('? for shortcuts');
		expect(result.matched).toBe(true);
		expect(result.pattern).toBe('codex-prompt');
	});

	it('matches codex waiting', () => {
		const result = detectNotification('What should Codex do?');
		expect(result.matched).toBe(true);
		expect(result.pattern).toBe('codex-waiting');
	});

	it('matches gemini prompt', () => {
		const result = detectNotification('>>> ');
		expect(result.matched).toBe(true);
		expect(result.pattern).toBe('gemini-prompt');
	});

	it('does not match unrelated text', () => {
		const result = detectNotification('compiling main.rs...');
		expect(result.matched).toBe(false);
		expect(result.pattern).toBe('');
	});

	it('matches custom patterns', () => {
		const custom = [{ name: 'custom-error', regex: /ERROR:/i }];
		const result = detectNotification('ERROR: something broke', custom);
		expect(result.matched).toBe(true);
		expect(result.pattern).toBe('custom-error');
	});

	it('prefers default patterns over custom', () => {
		const custom = [{ name: 'custom', regex: /Waiting/ }];
		const result = detectNotification('Waiting for input', custom);
		expect(result.matched).toBe(true);
		expect(result.pattern).toBe('claude-waiting');
	});
});

describe('createLineBuffer', () => {
	it('emits complete lines', () => {
		const lines: string[] = [];
		const buffer = createLineBuffer((line) => lines.push(line));
		buffer('hello\nworld\n');
		expect(lines).toEqual(['hello', 'world']);
	});

	it('buffers partial lines', () => {
		const lines: string[] = [];
		const buffer = createLineBuffer((line) => lines.push(line));
		buffer('hel');
		expect(lines).toEqual([]);
		buffer('lo\n');
		expect(lines).toEqual(['hello']);
	});

	it('handles multiple chunks forming lines', () => {
		const lines: string[] = [];
		const buffer = createLineBuffer((line) => lines.push(line));
		buffer('a');
		buffer('b\nc');
		buffer('d\n');
		expect(lines).toEqual(['ab', 'cd']);
	});

	it('handles empty input', () => {
		const lines: string[] = [];
		const buffer = createLineBuffer((line) => lines.push(line));
		buffer('');
		expect(lines).toEqual([]);
	});
});
