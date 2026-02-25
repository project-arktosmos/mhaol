<script lang="ts">
	import classNames from 'classnames';
	import { afterUpdate } from 'svelte';
	import { p2pService } from '$services/p2p.service';
	import { p2pAdapter } from '$adapters/classes/p2p.adapter';

	const state = p2pService.state;

	let messageInput = '';
	let chatContainer: HTMLDivElement;

	$: isConnected = $state.phase === 'connected';

	afterUpdate(() => {
		if (chatContainer) {
			chatContainer.scrollTop = chatContainer.scrollHeight;
		}
	});

	function handleSend() {
		if (!messageInput.trim() || !isConnected) return;
		p2pService.sendMessage(messageInput.trim());
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
			{#if $state.remoteAddress}
				<span class="font-mono text-xs text-base-content/50">
					Peer: {p2pAdapter.shortAddress($state.remoteAddress)}
				</span>
			{/if}
		</div>

		<!-- Messages -->
		<div
			bind:this={chatContainer}
			class="flex h-80 flex-col gap-2 overflow-y-auto rounded-lg bg-base-300 p-3"
		>
			{#if $state.messages.length === 0}
				<p class="text-center text-sm text-base-content/40">No messages yet</p>
			{:else}
				{#each $state.messages as msg (msg.id)}
					{@const isOwn = msg.address === $state.localAddress}
					<div class={classNames('chat', isOwn ? 'chat-end' : 'chat-start')}>
						<div class="chat-header text-xs text-base-content/50">
							{p2pAdapter.shortAddress(msg.address)}
							<time class="ml-1 opacity-50">{p2pAdapter.formatTimestamp(msg.timestamp)}</time>
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
				placeholder={isConnected ? 'Type a message...' : 'Waiting for connection...'}
				bind:value={messageInput}
				on:keydown={handleKeydown}
				disabled={!isConnected}
			/>
			<button
				class="btn btn-primary btn-sm"
				on:click={handleSend}
				disabled={!isConnected || !messageInput.trim()}
			>
				Send
			</button>
		</div>
	</div>
</div>
