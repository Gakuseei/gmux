export interface TerminalSession {
	id: string;
	name: string;
	shell: string;
	cwd: string;
	command?: string;
	bypassPermissions?: boolean;
	ptyId?: string;
	status: 'running' | 'needs-input' | 'ready' | 'exited';
	notificationCount: number;
}

export interface TerminalNode {
	type: 'terminal';
	terminalId: string;
}

export interface SplitBranchNode {
	type: 'split';
	direction: 'horizontal' | 'vertical';
	ratio: number;
	children: [SplitNode, SplitNode];
}

export type SplitNode = TerminalNode | SplitBranchNode;

export interface Workspace {
	id: string;
	name: string;
	folderId?: string;
	cwd: string;
	layout: SplitNode;
	sessions: TerminalSession[];
	createdAt: string;
}

export interface Folder {
	id: string;
	name: string;
	collapsed: boolean;
}

export type AppView = 'terminals' | 'insights';
export type InsightsTab = 'usage' | 'git' | 'info';
