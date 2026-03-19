<script lang="ts">
	import classNames from 'classnames';
	import type { LlmConversation } from '$types/llm.type';

	let {
		conversations,
		activeId,
		onSelect,
		onDelete,
		onCreate
	}: {
		conversations: LlmConversation[];
		activeId: string | null;
		onSelect: (id: string) => void;
		onDelete: (id: string) => void;
		onCreate: () => void;
	} = $props();
</script>

<div class="flex h-full flex-col">
	<div class="border-b border-base-300 p-2">
		<button class="btn w-full btn-sm btn-primary" onclick={onCreate}>New Chat</button>
	</div>

	<div class="flex-1 overflow-y-auto">
		{#if conversations.length === 0}
			<div class="p-4 text-center text-sm text-base-content/40">No conversations yet</div>
		{:else}
			<div class="flex flex-col gap-0.5 p-1">
				{#each conversations as conversation}
					<div
						class={classNames(
							'group flex cursor-pointer items-center justify-between rounded-lg px-3 py-2 hover:bg-base-200',
							{
								'bg-primary/10 font-medium': conversation.id === activeId
							}
						)}
						role="button"
						tabindex="0"
						onclick={() => onSelect(conversation.id)}
						onkeydown={(e: KeyboardEvent) => {
							if (e.key === 'Enter') onSelect(conversation.id);
						}}
					>
						<span class="truncate text-sm">{conversation.title}</span>
						<button
							class="btn opacity-0 btn-ghost btn-xs group-hover:opacity-100"
							onclick={(e: MouseEvent) => {
								e.stopPropagation();
								onDelete(conversation.id);
							}}
						>
							x
						</button>
					</div>
				{/each}
			</div>
		{/if}
	</div>
</div>
