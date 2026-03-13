const FOCUSABLE_SELECTOR = [
	'a[href]',
	'button:not([disabled])',
	'input:not([disabled])',
	'select:not([disabled])',
	'textarea:not([disabled])',
	'[tabindex]:not([tabindex="-1"])',
].join(', ');

export function createFocusTrap(container: HTMLElement): () => void {
	const previouslyFocused = document.activeElement as HTMLElement | null;

	function getFocusableElements(): HTMLElement[] {
		return Array.from(container.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTOR));
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key !== 'Tab') return;

		const focusable = getFocusableElements();
		if (focusable.length === 0) {
			e.preventDefault();
			return;
		}

		const first = focusable[0];
		const last = focusable[focusable.length - 1];

		if (e.shiftKey) {
			if (document.activeElement === first) {
				e.preventDefault();
				last.focus();
			}
		} else {
			if (document.activeElement === last) {
				e.preventDefault();
				first.focus();
			}
		}
	}

	container.addEventListener('keydown', handleKeydown);

	requestAnimationFrame(() => {
		const focusable = getFocusableElements();
		if (focusable.length > 0) {
			focusable[0].focus();
		}
	});

	return () => {
		container.removeEventListener('keydown', handleKeydown);
		if (previouslyFocused && typeof previouslyFocused.focus === 'function') {
			previouslyFocused.focus();
		}
	};
}
