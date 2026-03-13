<script lang="ts">
	interface Props {
		selected: string;
		onselect: (template: string) => void;
	}

	let { selected, onselect }: Props = $props();

	const templates = [
		{ id: 'single', label: '1 Terminal', slots: 1 },
		{ id: '2-side', label: '2 Terminals — side by side', slots: 2 },
		{ id: '2-vert', label: '2 Terminals — stacked', slots: 2 },
		{ id: '3-cols', label: '3 Terminals — columns', slots: 3 },
		{ id: '4-grid', label: '4 Terminals — 2x2 grid', slots: 4 },
		{ id: '6-grid', label: '6 Terminals — 3x2 grid', slots: 6 }
	];

	let description = $derived(templates.find((t) => t.id === selected)?.label ?? '');
</script>

<div class="layout-picker">
	<div class="template-grid">
		{#each templates as tmpl}
			<button
				class="template-btn"
				class:selected={selected === tmpl.id}
				onclick={() => onselect(tmpl.id)}
				title={tmpl.label}
			>
				<div class="preview preview-{tmpl.id}">
					{#if tmpl.id === 'single'}
						<div class="pane"></div>
					{:else if tmpl.id === '2-side'}
						<div class="pane"></div>
						<div class="pane"></div>
					{:else if tmpl.id === '2-vert'}
						<div class="pane"></div>
						<div class="pane"></div>
					{:else if tmpl.id === '3-cols'}
						<div class="pane"></div>
						<div class="pane"></div>
						<div class="pane"></div>
					{:else if tmpl.id === '4-grid'}
						<div class="pane"></div>
						<div class="pane"></div>
						<div class="pane"></div>
						<div class="pane"></div>
					{:else if tmpl.id === '6-grid'}
						<div class="pane"></div>
						<div class="pane"></div>
						<div class="pane"></div>
						<div class="pane"></div>
						<div class="pane"></div>
						<div class="pane"></div>
					{/if}
				</div>
			</button>
		{/each}
	</div>
	<div class="description">{description}</div>
</div>

<style>
	.layout-picker {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.template-grid {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: 8px;
	}

	.template-btn {
		background: var(--bg-primary);
		border: 2px solid var(--border-color);
		border-radius: var(--radius-button);
		padding: 10px;
		cursor: pointer;
		transition: border-color 0.15s;
	}

	.template-btn:hover {
		border-color: var(--text-secondary);
	}

	.template-btn.selected {
		border-color: var(--accent);
	}

	.preview {
		width: 100%;
		aspect-ratio: 16 / 10;
		display: grid;
		gap: 2px;
	}

	.pane {
		background: var(--accent);
		border-radius: 2px;
		opacity: 0.6;
	}

	.template-btn.selected .pane {
		opacity: 1;
	}

	.preview-single {
		grid-template-columns: 1fr;
	}

	.preview-2-side {
		grid-template-columns: 1fr 1fr;
	}

	.preview-2-vert {
		grid-template-rows: 1fr 1fr;
	}

	.preview-3-cols {
		grid-template-columns: 1fr 1fr 1fr;
	}

	.preview-4-grid {
		grid-template-columns: 1fr 1fr;
		grid-template-rows: 1fr 1fr;
	}

	.preview-6-grid {
		grid-template-columns: 1fr 1fr 1fr;
		grid-template-rows: 1fr 1fr;
	}

	.description {
		color: var(--text-secondary);
		font-size: 12px;
		text-align: center;
	}
</style>
