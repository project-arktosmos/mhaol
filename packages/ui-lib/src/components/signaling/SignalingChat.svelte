<script lang="ts">
	import classNames from 'classnames';
	import { signalingChatService } from 'frontend/services/signaling-chat.service';
	import { signalingAdapter } from 'frontend/adapters/classes/signaling.adapter';

	const chatStore = signalingChatService.state;

	let messageInput = $state('');
	let chatContainer: HTMLDivElement | undefined = $state();

	let isConnected = $derived($chatStore.phase === 'connected');
	let hasActivePeer = $derived($chatStore.activePeerId !== null);
	let activeChannelOpen = $derived(
		$chatStore.activePeerId !== null && $chatStore.peerIds.includes($chatStore.activePeerId)
	);
	let canSend = $derived(isConnected && activeChannelOpen);

	let filteredMessages = $derived(
		$chatStore.messages.filter((msg) => {
			if (msg.system) return true;
			if (!$chatStore.activePeerId) return false;
			return msg.address === $chatStore.activePeerId || msg.address === $chatStore.localPeerId;
		})
	);

	$effect(() => {
		if (filteredMessages.length && chatContainer) {
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

<div class="card flex h-full flex-col bg-base-200">
	<div class="card-body flex flex-1 flex-col gap-3 overflow-hidden p-4">
		<div class="flex items-center justify-between">
			<h2 class="card-title text-base">Chat</h2>
			{#if $chatStore.activePeerId}
				<span class="font-mono text-xs text-base-content/50">
					{signalingAdapter.shortAddress($chatStore.activePeerId)}
				</span>
			{/if}
		</div>

		<!-- Messages -->
		<div
			bind:this={chatContainer}
			class="flex flex-1 flex-col gap-2 overflow-y-auto rounded-lg bg-base-300 p-3"
		>
			{#if filteredMessages.length === 0}
				<p class="text-center text-sm text-base-content/40">
					{#if !isConnected}
						Connect to a signaling server to start chatting
					{:else if !hasActivePeer}
						Select a peer to start chatting
					{:else if !activeChannelOpen}
						Connecting to peer...
					{:else}
						No messages yet
					{/if}
				</p>
			{:else}
				{#each filteredMessages as msg (msg.id)}
					{#if msg.system}
						<div class="py-1 text-center text-xs text-base-content/40 italic">
							{msg.content}
						</div>
					{:else}
						{@const isOwn = msg.address === $chatStore.localPeerId}
						<div class={classNames('chat', isOwn ? 'chat-end' : 'chat-start')}>
							<div class="chat-header text-xs text-base-content/50">
								{signalingAdapter.shortAddress(msg.address)}
								<time class="ml-1 opacity-50"
									>{signalingAdapter.formatTimestamp(msg.timestamp)}</time
								>
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
					{/if}
				{/each}
			{/if}
		</div>

		<!-- Input -->
		<div class="flex gap-2">
			<input
				class="input-bordered input input-sm flex-1"
				type="text"
				placeholder={canSend ? 'Type a message...' : 'Select a peer to chat...'}
				bind:value={messageInput}
				onkeydown={handleKeydown}
				disabled={!canSend}
			/>
			<button
				class="btn btn-sm btn-primary"
				onclick={handleSend}
				disabled={!canSend || !messageInput.trim()}
			>
				Send
			</button>
		</div>
	</div>
</div>
