export interface StatusInfo {
	model?: string;
	reasoning?: string;
	inputTokens?: number;
	outputTokens?: number;
	contextPercent?: number;
	contextUsed?: number;
	contextTotal?: number;
}

const MODEL_PATTERNS = [
	/claude-(?:opus|sonnet|haiku)-[\d.-]+/i,
	/gpt-[\d.]+-\w+/i,
	/gemini-[\d.]+/i
];

const REASONING_PATTERNS: [RegExp, string | null][] = [
	[/Extra High/i, 'extra-high'],
	[/reasoning(?:_effort)?:\s*(high|medium|low)/i, null]
];

const INPUT_TOKENS = /(\d[\d,]+)\s*(?:input|↑)\s*tokens?/i;
const OUTPUT_TOKENS = /(\d[\d,]+)\s*(?:output|↓)\s*tokens?/i;
const CONTEXT_PERCENT = /(?:context|ctx)[:\s]*(\d+)%/i;
const CONTEXT_LEFT = /(\d+)% context left/i;
const CONTEXT_ABSOLUTE = /(\d+)k?\s*\/\s*(\d+)k/i;

function parseNum(s: string): number {
	return parseInt(s.replace(/,/g, ''), 10);
}

export function parseStatusLine(line: string): Partial<StatusInfo> {
	const result: Partial<StatusInfo> = {};

	for (const pat of MODEL_PATTERNS) {
		const m = line.match(pat);
		if (m) {
			result.model = m[0];
			break;
		}
	}

	for (const [pat, fixed] of REASONING_PATTERNS) {
		const m = line.match(pat);
		if (m) {
			result.reasoning = fixed ?? m[1].toLowerCase();
			break;
		}
	}

	const inputMatch = line.match(INPUT_TOKENS);
	if (inputMatch) result.inputTokens = parseNum(inputMatch[1]);

	const outputMatch = line.match(OUTPUT_TOKENS);
	if (outputMatch) result.outputTokens = parseNum(outputMatch[1]);

	const ctxPct = line.match(CONTEXT_PERCENT);
	if (ctxPct) result.contextPercent = parseInt(ctxPct[1], 10);

	const ctxLeft = line.match(CONTEXT_LEFT);
	if (ctxLeft) result.contextPercent = 100 - parseInt(ctxLeft[1], 10);

	const ctxAbs = line.match(CONTEXT_ABSOLUTE);
	if (ctxAbs) {
		const used = ctxAbs[1].endsWith('k') ? parseNum(ctxAbs[1]) * 1000 : parseNum(ctxAbs[1]);
		const total = ctxAbs[2].endsWith('k') ? parseNum(ctxAbs[2]) * 1000 : parseNum(ctxAbs[2]);
		result.contextUsed = used;
		result.contextTotal = total;
	}

	return result;
}
