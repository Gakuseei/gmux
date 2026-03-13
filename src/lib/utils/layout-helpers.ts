import type { SplitNode, TerminalSession } from '$lib/types/workspace';

export function splitNode(
	layout: SplitNode,
	terminalId: string,
	direction: 'horizontal' | 'vertical',
	newSession: TerminalSession
): SplitNode {
	if (layout.type === 'terminal') {
		if (layout.terminalId === terminalId) {
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
		children: layout.children?.map((child) => splitNode(child, terminalId, direction, newSession))
	};
}

export function removeNode(layout: SplitNode, terminalId: string): SplitNode | null {
	if (layout.type === 'terminal') {
		return layout.terminalId === terminalId ? null : { ...layout };
	}

	if (!layout.children || layout.children.length !== 2) return { ...layout };

	const newChildren = layout.children.map((child) => removeNode(child, terminalId));

	if (newChildren[0] === null && newChildren[1] === null) return null;
	if (newChildren[0] === null) return newChildren[1];
	if (newChildren[1] === null) return newChildren[0];

	return {
		...layout,
		children: newChildren as SplitNode[]
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
