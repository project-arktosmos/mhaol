<script lang="ts">
	import classNames from 'classnames';
	import type { ChatMessage } from '$types/llm.type';

	let {
		message,
		isStreaming = false
	}: {
		message: ChatMessage;
		isStreaming?: boolean;
	} = $props();

	let chatClass = $derived(
		classNames('chat', {
			'chat-start': message.role === 'assistant' || message.role === 'system',
			'chat-end': message.role === 'user'
		})
	);

	let bubbleClass = $derived(
		classNames('chat-bubble whitespace-pre-wrap', {
			'chat-bubble-primary': message.role === 'user',
			'chat-bubble-neutral': message.role === 'assistant',
			'chat-bubble-info text-xs opacity-70': message.role === 'system'
		})
	);

	let roleLabel = $derived(
		message.role === 'user' ? 'You' : message.role === 'assistant' ? 'Assistant' : 'System'
	);
</script>

<div class={chatClass}>
	<div class="chat-header text-xs opacity-60">{roleLabel}</div>
	<div class={bubbleClass}>
		{message.content}{#if isStreaming}<span class="animate-pulse">|</span>{/if}
	</div>
</div>
