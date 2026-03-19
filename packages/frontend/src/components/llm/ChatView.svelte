<script lang="ts">
	import ChatBubble from 'frontend/components/llm/ChatBubble.svelte';
	import type { ChatMessage } from 'frontend/types/llm.type';

	let {
		messages,
		streamingContent,
		isGenerating,
		onSendMessage,
		onCancelGeneration
	}: {
		messages: ChatMessage[];
		streamingContent: string;
		isGenerating: boolean;
		onSendMessage: (content: string) => void;
		onCancelGeneration: () => void;
	} = $props();

	let input = $state('');
	let messagesEnd: HTMLDivElement | undefined = $state();

	$effect(() => {
		if (messages.length || streamingContent) {
			messagesEnd?.scrollIntoView({ behavior: 'smooth' });
		}
	});

	function handleSend() {
		const trimmed = input.trim();
		if (!trimmed || isGenerating) return;
		input = '';
		onSendMessage(trimmed);
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter' && !e.shiftKey) {
			e.preventDefault();
			handleSend();
		}
	}
</script>

<div class="flex h-full flex-col">
	<div class="flex-1 overflow-y-auto p-4">
		{#if messages.length === 0 && !streamingContent}
			<div class="flex h-full items-center justify-center text-base-content/40">
				<p>Send a message to start chatting</p>
			</div>
		{:else}
			{#each messages as message}
				<ChatBubble {message} />
			{/each}

			{#if streamingContent}
				<ChatBubble message={{ role: 'assistant', content: streamingContent }} isStreaming={true} />
			{/if}
		{/if}
		<div bind:this={messagesEnd}></div>
	</div>

	<div class="border-t border-base-300 p-3">
		<div class="flex gap-2">
			<textarea
				class="textarea-bordered textarea flex-1 resize-none"
				rows={2}
				placeholder="Type a message..."
				bind:value={input}
				onkeydown={handleKeydown}
				disabled={isGenerating}
			></textarea>
			<div class="flex flex-col gap-1">
				{#if isGenerating}
					<button class="btn btn-sm btn-error" onclick={onCancelGeneration}>Cancel</button>
				{:else}
					<button class="btn btn-sm btn-primary" onclick={handleSend} disabled={!input.trim()}>
						Send
					</button>
				{/if}
			</div>
		</div>
	</div>
</div>
