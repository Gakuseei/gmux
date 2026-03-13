interface DetectionResult {
	matched: boolean;
	pattern: string;
}

interface PatternDef {
	name: string;
	regex: RegExp;
}

const DEFAULT_PATTERNS: PatternDef[] = [
	{ name: 'claude-prompt', regex: /[❯›]\s*$/ },
	{ name: 'claude-confirm', regex: /\?\s*\(y\/n\)/i },
	{ name: 'claude-waiting', regex: /waiting for/i },
	{ name: 'claude-permission', regex: /\(y\) to approve/i },
	{ name: 'codex-prompt', regex: /\? for shortcuts/ },
	{ name: 'codex-waiting', regex: /what should codex do/i },
	{ name: 'gemini-prompt', regex: />>>\s*$/ }
];

export function detectNotification(
	line: string,
	customPatterns?: PatternDef[]
): DetectionResult {
	const patterns = customPatterns ? [...DEFAULT_PATTERNS, ...customPatterns] : DEFAULT_PATTERNS;

	for (const p of patterns) {
		if (p.regex.test(line)) {
			return { matched: true, pattern: p.name };
		}
	}

	return { matched: false, pattern: '' };
}

export function createLineBuffer(onLine: (line: string) => void): (chunk: string) => void {
	let buffer = '';

	return (chunk: string) => {
		buffer += chunk;
		const lines = buffer.split('\n');
		buffer = lines.pop()!;
		for (const line of lines) {
			onLine(line);
		}
	};
}
