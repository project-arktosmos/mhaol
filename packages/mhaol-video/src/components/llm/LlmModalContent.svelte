<script lang="ts">
	import { llmService } from '$services/llm.service';
	import ConversationList from '$components/llm/ConversationList.svelte';
	import ChatView from '$components/llm/ChatView.svelte';
	import ModelManager from '$components/llm/ModelManager.svelte';

	let activeTab: 'chat' | 'models' = $state('chat');

	const store = llmService.store;

	let status = $derived($store.status);
	let models = $derived($store.models);
	let conversations = $derived($store.conversations);
	let activeConversationId = $derived($store.activeConversationId);
	let messages = $derived($store.messages);
	let streamingContent = $derived($store.streamingContent);
	let isGenerating = $derived($store.isGenerating);
	let downloadProgress = $derived($store.downloadProgress);
	let loading = $derived($store.loading);

	$effect(() => {
		llmService.initialize();
	});

	function handleCreateConversation() {
		const title = `Chat ${conversations.length + 1}`;
		llmService.createConversation(title);
	}
</script>

<div class="flex h-[70vh] flex-col">
	<div class="flex items-center justify-between border-b border-base-300 px-4 py-2">
		<h2 class="text-lg font-bold">Local LLM</h2>
		<div role="tablist" class="tabs-boxed tabs tabs-sm">
			<button
				role="tab"
				class="tab"
				class:tab-active={activeTab === 'chat'}
				onclick={() => (activeTab = 'chat')}
			>
				Chat
			</button>
			<button
				role="tab"
				class="tab"
				class:tab-active={activeTab === 'models'}
				onclick={() => (activeTab = 'models')}
			>
				Models
			</button>
		</div>
	</div>

	{#if activeTab === 'models'}
		<div class="flex-1 overflow-y-auto p-4">
			<ModelManager
				{status}
				{models}
				{downloadProgress}
				{loading}
				onLoadModel={(fileName) => llmService.loadModel(fileName)}
				onUnloadModel={() => llmService.unloadModel()}
				onDownloadModel={(repoId, fileName) => llmService.downloadModel(repoId, fileName)}
			/>
		</div>
	{:else}
		<div class="flex flex-1 overflow-hidden">
			<div class="w-56 shrink-0 border-r border-base-300">
				<ConversationList
					{conversations}
					activeId={activeConversationId}
					onSelect={(id) => llmService.selectConversation(id)}
					onDelete={(id) => llmService.deleteConversation(id)}
					onCreate={handleCreateConversation}
				/>
			</div>

			<div class="flex-1">
				{#if activeConversationId}
					<ChatView
						{messages}
						{streamingContent}
						{isGenerating}
						onSendMessage={(content) => llmService.sendMessage(content)}
						onCancelGeneration={() => llmService.cancelGeneration()}
					/>
				{:else}
					<div class="flex h-full items-center justify-center text-base-content/40">
						<p>Select or create a conversation to begin</p>
					</div>
				{/if}
			</div>
		</div>
	{/if}
</div>
