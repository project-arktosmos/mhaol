<script lang="ts">
	import classNames from 'classnames';
	import { signalingChatService } from '$services/signaling-chat.service';
	import { signalingAdapter } from '$adapters/classes/signaling.adapter';

	const chatStore = signalingChatService.state;

	let messageInput = $state('');
	let chatContainer: HTMLDivElement | undefined = $state();

	let isConnected = $derived($chatStore.phase === 'connected');
	let hasOpenChannels = $derived($chatStore.peerIds.length > 0);
	let canSend = $derived(isConnected && hasOpenChannels);

	$effect(() => {
		if ($chatStore.messages.length && chatContainer) {
			chatContainer.scrollTop = chatContainer.scrollHeight;
		}
	});

	function handleSend() {
		if (!messageInput.trim() || !canSend) return;
		signalingChatService.sendMessage(messageInput.trim());
		messageInput = '';
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter' && !event.shiftKey) {
			event.preventDefault();
			handleSend();
		}
	}
</script>

<div class="card bg-base-200 flex flex-col">
	<div class="card-body gap-3 p-4">
		<div class="flex items-center justify-between">
			<h2 class="card-title text-base">Chat</h2>
			{#if $chatStore.peerIds.length > 0}
				<span class="text-xs text-base-content/50">
					{$chatStore.peerIds.length} peer{$chatStore.peerIds.length !== 1 ? 's' : ''}
				</span>
			{/if}
		</div>

		<!-- Messages -->
		<div
			bind:this={chatContainer}
			class="flex h-80 flex-col gap-2 overflow-y-auto rounded-lg bg-base-300 p-3"
		>
			{#if $chatStore.messages.length === 0}
				<p class="text-center text-sm text-base-content/40">
					{#if !isConnected}
						Connect to a signaling server to start chatting
					{:else if !hasOpenChannels}
						Waiting for peers to join...
					{:else}
						No messages yet
					{/if}
				</p>
			{:else}
				{#each $chatStore.messages as msg (msg.id)}
					{@const isOwn = msg.address === $chatStore.localPeerId}
					<div class={classNames('chat', isOwn ? 'chat-end' : 'chat-start')}>
						<div class="chat-header text-xs text-base-content/50">
							{signalingAdapter.shortAddress(msg.address)}
							<time class="ml-1 opacity-50">{signalingAdapter.formatTimestamp(msg.timestamp)}</time>
						</div>
						<div
							class={classNames('chat-bubble', {
								'chat-bubble-primary': isOwn,
								'chat-bubble-neutral': !isOwn
							})}
						>
							{msg.content}
						</div>
					</div>
				{/each}
			{/if}
		</div>

		<!-- Input -->
		<div class="flex gap-2">
			<input
				class="input input-bordered input-sm flex-1"
				type="text"
				placeholder={canSend ? 'Type a message...' : 'Waiting for connection...'}
				bind:value={messageInput}
				onkeydown={handleKeydown}
				disabled={!canSend}
			/>
			<button
				class="btn btn-primary btn-sm"
				onclick={handleSend}
				disabled={!canSend || !messageInput.trim()}
			>
				Send
			</button>
		</div>
	</div>
</div>
