<script lang="ts">
	import { appStore } from '$lib/stores/app.svelte';
	import { recentPathsStore } from '$lib/stores/recent-paths.svelte';
	import { generateLayout } from '$lib/utils/layout-helpers';
	import PathPicker from './PathPicker.svelte';
	import LayoutPicker from './LayoutPicker.svelte';
	import AgentPicker from './AgentPicker.svelte';
	import type { TerminalSession, Workspace } from '$lib/types/workspace';

	interface AgentConfig {
		type: 'claude' | 'codex' | 'gemini' | 'shell';
		label: string;
		command: string;
		count: number;
		bypassPermissions?: boolean;
	}

	const TEMPLATE_SLOTS: Record<string, number> = {
		single: 1,
		'2-side': 2,
		'2-vert': 2,
		'3-cols': 3,
		'4-grid': 4,
		'6-grid': 6
	};

	let name = $state('');
	let cwd = $state('');
	let selectedLayout = $state('single');
	let agents = $state<AgentConfig[]>([
		{ type: 'claude', label: 'Claude Code', command: 'claude', count: 0, bypassPermissions: false },
		{ type: 'codex', label: 'Codex CLI', command: 'codex', count: 0 },
		{ type: 'gemini', label: 'Gemini CLI', command: 'gemini', count: 0 },
		{ type: 'shell', label: 'Shell', command: '', count: 0 }
	]);

	let totalSlots = $derived(TEMPLATE_SLOTS[selectedLayout] ?? 1);

	function resetAgentCounts() {
		agents = agents.map((a) => ({ ...a, count: 0 }));
	}

	function handleLayoutSelect(template: string) {
		selectedLayout = template;
		const newSlots = TEMPLATE_SLOTS[template] ?? 1;
		const assigned = agents.reduce((sum, a) => sum + a.count, 0);
		if (assigned > newSlots) {
			resetAgentCounts();
		}
	}

	function handleAgentsChange(updated: AgentConfig[]) {
		agents = updated;
	}

	function buildCommand(agent: AgentConfig): string {
		switch (agent.type) {
			case 'claude':
				return agent.bypassPermissions ? 'claude --dangerously-skip-permissions' : 'claude';
			case 'codex':
				return 'codex --dangerously-bypass-approvals-and-sandbox';
			case 'gemini':
				return 'gemini';
			default:
				return '';
		}
	}

	function launch() {
		const sessions: TerminalSession[] = [];

		for (const agent of agents) {
			for (let i = 0; i < agent.count; i++) {
				sessions.push({
					id: crypto.randomUUID(),
					name: agent.count > 1 ? `${agent.label} ${i + 1}` : agent.label,
					shell: '',
					cwd: cwd || '~',
					command: buildCommand(agent) || undefined,
					bypassPermissions: agent.type === 'claude' ? agent.bypassPermissions : undefined,
					status: 'ready',
					notificationCount: 0
				});
			}
		}

		const remaining = totalSlots - sessions.length;
		for (let i = 0; i < remaining; i++) {
			sessions.push({
				id: crypto.randomUUID(),
				name: `Shell ${sessions.length + 1}`,
				shell: '',
				cwd: cwd || '~',
				status: 'ready',
				notificationCount: 0
			});
		}

		const layout = generateLayout(selectedLayout, sessions);

		const workspace: Workspace = {
			id: crypto.randomUUID(),
			name: name || 'Untitled',
			cwd: cwd || '~',
			layout,
			sessions,
			createdAt: new Date().toISOString()
		};

		appStore.addWorkspace(workspace);

		if (cwd) {
			recentPathsStore.addPath(cwd);
		}

		close();
	}

	function close() {
		appStore.showNewWorkspaceModal = false;
		name = '';
		cwd = '';
		selectedLayout = 'single';
		resetAgentCounts();
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') close();
	}

	function handleBackdropClick(e: MouseEvent) {
		if (e.target === e.currentTarget) close();
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="backdrop" onclick={handleBackdropClick} onkeydown={handleKeydown}>
	<!-- svelte-ignore a11y_interactive_supports_focus -->
	<div class="modal" role="dialog" onclick={(e) => e.stopPropagation()} onkeydown={(e) => e.stopPropagation()}>
		<div class="modal-header">
			<h2>New Workspace</h2>
			<button class="close-btn" onclick={close}>&times;</button>
		</div>

		<div class="modal-body">
			<div class="field">
				<label class="field-label" for="ws-name">Name</label>
				<input
					id="ws-name"
					type="text"
					bind:value={name}
					placeholder="My Workspace"
					class="text-input"
				/>
			</div>

			<div class="field">
				<span class="field-label">Working Directory</span>
				<PathPicker value={cwd} onchange={(v) => (cwd = v)} />
			</div>

			<div class="field">
				<span class="field-label">Layout</span>
				<LayoutPicker selected={selectedLayout} onselect={handleLayoutSelect} />
			</div>

			<div class="field">
				<AgentPicker
					{totalSlots}
					{agents}
					onchange={handleAgentsChange}
				/>
			</div>
		</div>

		<div class="modal-footer">
			<button class="btn btn-cancel" onclick={close}>Cancel</button>
			<button class="btn btn-launch" onclick={launch}>Launch</button>
		</div>
	</div>
</div>

<style>
	.backdrop {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.6);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1000;
		border: none;
		cursor: default;
		padding: 0;
	}

	.modal {
		background: var(--bg-surface);
		border-radius: var(--radius-container);
		width: 100%;
		max-width: 600px;
		max-height: 90vh;
		overflow-y: auto;
		display: flex;
		flex-direction: column;
		cursor: auto;
	}

	.modal-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 16px 20px;
		border-bottom: 1px solid var(--border-color);
	}

	.modal-header h2 {
		font-size: 16px;
		font-weight: 600;
		color: var(--text-primary);
	}

	.close-btn {
		background: none;
		border: none;
		color: var(--text-secondary);
		font-size: 20px;
		cursor: pointer;
		padding: 0 4px;
		line-height: 1;
	}

	.close-btn:hover {
		color: var(--text-primary);
	}

	.modal-body {
		padding: 20px;
		display: flex;
		flex-direction: column;
		gap: 18px;
	}

	.field {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.field-label {
		font-size: 12px;
		font-weight: 500;
		color: var(--text-secondary);
		text-transform: uppercase;
		letter-spacing: 0.5px;
	}

	.text-input {
		background: var(--bg-primary);
		border: 1px solid var(--border-color);
		border-radius: var(--radius-button);
		color: var(--text-primary);
		padding: 8px 10px;
		font-size: 13px;
		outline: none;
	}

	.text-input:focus {
		border-color: var(--accent);
	}

	.modal-footer {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
		padding: 16px 20px;
		border-top: 1px solid var(--border-color);
	}

	.btn {
		padding: 8px 18px;
		border-radius: var(--radius-button);
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
		border: none;
	}

	.btn-cancel {
		background: var(--bg-primary);
		color: var(--text-secondary);
		border: 1px solid var(--border-color);
	}

	.btn-cancel:hover {
		color: var(--text-primary);
		border-color: var(--text-secondary);
	}

	.btn-launch {
		background: var(--accent);
		color: #fff;
	}

	.btn-launch:hover {
		opacity: 0.9;
	}
</style>
