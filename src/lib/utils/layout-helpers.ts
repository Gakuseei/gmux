import type { SplitNode, TerminalSession } from '$lib/types/workspace';

export function findFirstTerminalId(node: SplitNode): string | null {
	if (node.type === 'terminal') return node.terminalId;
	for (const child of node.children) {
		const id = findFirstTerminalId(child);
		if (id) return id;
	}
	return null;
}

const MAX_SPLIT_DEPTH = 8;

export function splitNode(
	layout: SplitNode,
	terminalId: string,
	direction: 'horizontal' | 'vertical',
	newSession: TerminalSession,
	depth: number = 0
): SplitNode {
	if (layout.type === 'terminal') {
		if (layout.terminalId === terminalId) {
			if (depth >= MAX_SPLIT_DEPTH) return { ...layout };
			return {
				type: 'split',
				direction,
				ratio: 0.5,
				children: [
					{ ...layout },
					{ type: 'terminal', terminalId: newSession.id }
				]
			};
		}
		return { ...layout };
	}

	return {
		...layout,
		children: [
			splitNode(layout.children[0], terminalId, direction, newSession, depth + 1),
			splitNode(layout.children[1], terminalId, direction, newSession, depth + 1)
		]
	};
}

export function removeNode(layout: SplitNode, terminalId: string): SplitNode | null {
	if (layout.type === 'terminal') {
		return layout.terminalId === terminalId ? null : { ...layout };
	}

	const newChildren = layout.children.map((child) => removeNode(child, terminalId));

	if (newChildren[0] === null && newChildren[1] === null) return null;
	if (newChildren[0] === null) return newChildren[1];
	if (newChildren[1] === null) return newChildren[0];

	return {
		...layout,
		children: newChildren as [SplitNode, SplitNode]
	};
}

export function generateLayout(template: string, sessions: TerminalSession[]): SplitNode {
	const node = (index: number): SplitNode => ({
		type: 'terminal',
		terminalId: sessions[index]?.id ?? ''
	});

	switch (template) {
		case '2-side':
			return {
				type: 'split',
				direction: 'horizontal',
				ratio: 0.5,
				children: [node(0), node(1)]
			};

		case '2-vert':
			return {
				type: 'split',
				direction: 'vertical',
				ratio: 0.5,
				children: [node(0), node(1)]
			};

		case '4-grid':
			return {
				type: 'split',
				direction: 'vertical',
				ratio: 0.5,
				children: [
					{
						type: 'split',
						direction: 'horizontal',
						ratio: 0.5,
						children: [node(0), node(1)]
					},
					{
						type: 'split',
						direction: 'horizontal',
						ratio: 0.5,
						children: [node(2), node(3)]
					}
				]
			};

		case '3-cols':
			return {
				type: 'split',
				direction: 'horizontal',
				ratio: 0.333,
				children: [
					node(0),
					{
						type: 'split',
						direction: 'horizontal',
						ratio: 0.5,
						children: [node(1), node(2)]
					}
				]
			};

		case '6-grid':
			return {
				type: 'split',
				direction: 'vertical',
				ratio: 0.5,
				children: [
					{
						type: 'split',
						direction: 'horizontal',
						ratio: 0.333,
						children: [
							node(0),
							{
								type: 'split',
								direction: 'horizontal',
								ratio: 0.5,
								children: [node(1), node(2)]
							}
						]
					},
					{
						type: 'split',
						direction: 'horizontal',
						ratio: 0.333,
						children: [
							node(3),
							{
								type: 'split',
								direction: 'horizontal',
								ratio: 0.5,
								children: [node(4), node(5)]
							}
						]
					}
				]
			};

		default:
			return node(0);
	}
}
