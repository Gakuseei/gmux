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

const NOTIFICATION_KEYWORDS = /[❯›?]|waiting|permission|approve|shortcuts|codex|>>>/i;

export function detectNotification(
	line: string,
	customPatterns?: PatternDef[]
): DetectionResult {
	if (!customPatterns && !NOTIFICATION_KEYWORDS.test(line)) {
		return { matched: false, pattern: '' };
	}

	const patterns = customPatterns ? [...DEFAULT_PATTERNS, ...customPatterns] : DEFAULT_PATTERNS;

	for (const p of patterns) {
		if (p.regex.test(line)) {
			return { matched: true, pattern: p.name };
		}
	}

	return { matched: false, pattern: '' };
}

const FLUSH_TIMEOUT_MS = 150;

export function createLineBuffer(onLine: (line: string) => void): (chunk: string) => void {
	let buffer = '';
	let flushTimer: ReturnType<typeof setTimeout> | null = null;

	return (chunk: string) => {
		if (flushTimer) {
			clearTimeout(flushTimer);
			flushTimer = null;
		}

		buffer += chunk;
		const lines = buffer.split('\n');
		buffer = lines.pop()!;
		for (const line of lines) {
			onLine(line);
		}

		if (buffer.length > 0) {
			flushTimer = setTimeout(() => {
				if (buffer.length > 0) {
					onLine(buffer);
					buffer = '';
				}
				flushTimer = null;
			}, FLUSH_TIMEOUT_MS);
		}
	};
}
