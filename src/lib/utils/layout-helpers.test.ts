import { describe, it, expect } from 'vitest';
import { generateLayout, splitNode, removeNode } from './layout-helpers';
import type { TerminalSession, SplitNode } from '$lib/types/workspace';

function makeSession(id: string): TerminalSession {
	return {
		id,
		name: id,
		shell: '/bin/bash',
		cwd: '/tmp',
		status: 'running',
		notificationCount: 0
	};
}

describe('generateLayout', () => {
	it('generates single terminal layout', () => {
		const sessions = [makeSession('a')];
		const layout = generateLayout('single', sessions);
		expect(layout.type).toBe('terminal');
		expect(layout.terminalId).toBe('a');
	});

	it('generates 2-side layout', () => {
		const sessions = [makeSession('a'), makeSession('b')];
		const layout = generateLayout('2-side', sessions);
		expect(layout.type).toBe('split');
		expect(layout.direction).toBe('horizontal');
		expect(layout.children).toHaveLength(2);
		expect(layout.children![0].terminalId).toBe('a');
		expect(layout.children![1].terminalId).toBe('b');
	});

	it('generates 2-vert layout', () => {
		const sessions = [makeSession('a'), makeSession('b')];
		const layout = generateLayout('2-vert', sessions);
		expect(layout.type).toBe('split');
		expect(layout.direction).toBe('vertical');
	});

	it('generates 4-grid layout', () => {
		const sessions = [makeSession('a'), makeSession('b'), makeSession('c'), makeSession('d')];
		const layout = generateLayout('4-grid', sessions);
		expect(layout.type).toBe('split');
		expect(layout.direction).toBe('vertical');
		expect(layout.children).toHaveLength(2);
		expect(layout.children![0].type).toBe('split');
		expect(layout.children![1].type).toBe('split');
	});

	it('generates 3-cols layout', () => {
		const sessions = [makeSession('a'), makeSession('b'), makeSession('c')];
		const layout = generateLayout('3-cols', sessions);
		expect(layout.type).toBe('split');
		expect(layout.direction).toBe('horizontal');
	});

	it('falls back to single for unknown template', () => {
		const sessions = [makeSession('a')];
		const layout = generateLayout('unknown', sessions);
		expect(layout.type).toBe('terminal');
	});
});

describe('splitNode', () => {
	it('splits a terminal node horizontally', () => {
		const layout: SplitNode = { type: 'terminal', terminalId: 'a' };
		const newSession = makeSession('b');
		const result = splitNode(layout, 'a', 'horizontal', newSession);
		expect(result.type).toBe('split');
		expect(result.direction).toBe('horizontal');
		expect(result.children).toHaveLength(2);
		expect(result.children![0].terminalId).toBe('a');
		expect(result.children![1].terminalId).toBe('b');
	});

	it('splits a terminal node vertically', () => {
		const layout: SplitNode = { type: 'terminal', terminalId: 'a' };
		const newSession = makeSession('b');
		const result = splitNode(layout, 'a', 'vertical', newSession);
		expect(result.type).toBe('split');
		expect(result.direction).toBe('vertical');
	});

	it('does not split non-matching terminal', () => {
		const layout: SplitNode = { type: 'terminal', terminalId: 'a' };
		const newSession = makeSession('c');
		const result = splitNode(layout, 'b', 'horizontal', newSession);
		expect(result.type).toBe('terminal');
		expect(result.terminalId).toBe('a');
	});

	it('recursively finds and splits the target', () => {
		const layout: SplitNode = {
			type: 'split',
			direction: 'horizontal',
			children: [
				{ type: 'terminal', terminalId: 'a' },
				{ type: 'terminal', terminalId: 'b' }
			]
		};
		const newSession = makeSession('c');
		const result = splitNode(layout, 'b', 'vertical', newSession);
		expect(result.children![1].type).toBe('split');
		expect(result.children![1].direction).toBe('vertical');
	});
});

describe('removeNode', () => {
	it('removes a matching terminal node', () => {
		const layout: SplitNode = { type: 'terminal', terminalId: 'a' };
		const result = removeNode(layout, 'a');
		expect(result).toBeNull();
	});

	it('keeps a non-matching terminal node', () => {
		const layout: SplitNode = { type: 'terminal', terminalId: 'a' };
		const result = removeNode(layout, 'b');
		expect(result).not.toBeNull();
		expect(result!.terminalId).toBe('a');
	});

	it('collapses split when one child is removed', () => {
		const layout: SplitNode = {
			type: 'split',
			direction: 'horizontal',
			children: [
				{ type: 'terminal', terminalId: 'a' },
				{ type: 'terminal', terminalId: 'b' }
			]
		};
		const result = removeNode(layout, 'a');
		expect(result).not.toBeNull();
		expect(result!.type).toBe('terminal');
		expect(result!.terminalId).toBe('b');
	});

	it('returns null when both children are removed', () => {
		const layout: SplitNode = {
			type: 'split',
			direction: 'horizontal',
			children: [
				{ type: 'terminal', terminalId: 'a' },
				{ type: 'terminal', terminalId: 'a' }
			]
		};
		const result = removeNode(layout, 'a');
		expect(result).toBeNull();
	});
});
