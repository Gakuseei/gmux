export function formatTokens(n: number, precision: 'compact' | 'detailed' = 'compact'): string {
	if (n >= 1_000_000) return (n / 1_000_000).toFixed(precision === 'detailed' ? 2 : 1) + 'M';
	if (n >= 1_000) return precision === 'detailed' ? (n / 1_000).toFixed(1) + 'k' : Math.round(n / 1_000) + 'k';
	return String(n);
}
