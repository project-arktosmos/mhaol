<script lang="ts">
	import classNames from 'classnames';
	import { page } from '$app/stores';
	import { onMount } from 'svelte';
	import { cloudItemService } from 'frontend/services/cloud-item.service';
	import { cloudAdapter } from 'frontend/adapters/classes/cloud.adapter';

	const iState = cloudItemService.state;

	let itemId = $derived($page.params.id ?? '');
	let svc = $derived($iState);
	let item = $derived(svc.currentItem);

	let newAttrKey = $state('');
	let newAttrValue = $state('');
	let newAttrType = $state('string');

	onMount(async () => {
		if (itemId) await cloudItemService.getItem(itemId);
	});

	async function handleAddAttribute() {
		if (!newAttrKey || !newAttrValue || !itemId) return;
		await cloudItemService.setAttribute(itemId, newAttrKey, newAttrValue, newAttrType);
		newAttrKey = '';
		newAttrValue = '';
		newAttrType = 'string';
	}

	async function handleRemoveAttribute(key: string) {
		if (itemId) await cloudItemService.removeAttribute(itemId, key);
	}
</script>

<div class="p-6">
	<div class="mb-4">
		<a href="/" class="btn btn-ghost btn-sm">Back to Dashboard</a>
	</div>

	{#if svc.loading}
		<div class="flex justify-center py-20">
			<span class="loading loading-lg loading-spinner"></span>
		</div>
	{:else if item}
		<div class="grid grid-cols-1 gap-6 lg:grid-cols-3">
			<div class="lg:col-span-2">
				<div class="card bg-base-100 shadow-md">
					<div class="card-body">
						<h2 class="card-title">{item.filename}</h2>
						<p class="text-sm break-all text-base-content/60">{item.path}</p>

						<div class="mt-3 flex flex-wrap gap-2">
							<span class={classNames('badge', cloudAdapter.extensionBadgeClass(item.extension))}>
								{item.extension}
							</span>
							{#if item.sizeBytes}
								<span class="badge badge-neutral">
									{cloudAdapter.formatBytes(item.sizeBytes)}
								</span>
							{/if}
							{#if item.mimeType}
								<span class="badge badge-ghost">{item.mimeType}</span>
							{/if}
						</div>
					</div>
				</div>

				<div class="card mt-4 bg-base-100 shadow-md">
					<div class="card-body">
						<h3 class="card-title text-lg">Attributes</h3>

						<div class="overflow-x-auto">
							<table class="table table-sm">
								<thead>
									<tr>
										<th>Key</th>
										<th>Value</th>
										<th>Type</th>
										<th>Source</th>
										<th></th>
									</tr>
								</thead>
								<tbody>
									{#each item.attributes as attr (attr.key + attr.source)}
										<tr>
											<td class="font-medium">{attr.key}</td>
											<td>{cloudAdapter.formatAttributeValue(attr.value, attr.typeId)}</td>
											<td>
												<span
													class={classNames(
														'badge badge-xs',
														cloudAdapter.attributeTypeBadgeClass(attr.typeId)
													)}
												>
													{attr.typeId}
												</span>
											</td>
											<td>
												<span class="badge badge-ghost badge-xs">{attr.source}</span>
											</td>
											<td>
												<button
													class="btn text-error btn-ghost btn-xs"
													onclick={() => handleRemoveAttribute(attr.key)}
												>
													Remove
												</button>
											</td>
										</tr>
									{/each}
								</tbody>
							</table>
						</div>

						{#if item.attributes.length === 0}
							<p class="py-4 text-center text-sm text-base-content/60">No attributes yet</p>
						{/if}

						<div class="divider"></div>

						<h4 class="text-sm font-semibold">Add Attribute</h4>
						<div class="flex flex-wrap gap-2">
							<input
								type="text"
								class="input-bordered input input-sm w-32"
								placeholder="Key"
								bind:value={newAttrKey}
							/>
							<input
								type="text"
								class="input-bordered input input-sm w-48"
								placeholder="Value"
								bind:value={newAttrValue}
							/>
							<select class="select-bordered select select-sm" bind:value={newAttrType}>
								<option value="string">String</option>
								<option value="number">Number</option>
								<option value="boolean">Boolean</option>
								<option value="date">Date</option>
								<option value="url">URL</option>
								<option value="duration">Duration</option>
								<option value="bytes">Bytes</option>
								<option value="tags">Tags</option>
								<option value="json">JSON</option>
							</select>
							<button
								class="btn btn-sm btn-primary"
								disabled={!newAttrKey || !newAttrValue}
								onclick={handleAddAttribute}
							>
								Add
							</button>
						</div>
					</div>
				</div>
			</div>

			<div>
				{#if item.links.length > 0}
					<div class="card bg-base-100 shadow-md">
						<div class="card-body">
							<h3 class="card-title text-lg">External Links</h3>
							<div class="space-y-2">
								{#each item.links as link (link.service)}
									<div class="flex items-center justify-between">
										<span class="badge badge-primary">{link.service}</span>
										<span class="text-sm text-base-content/60">{link.serviceId}</span>
									</div>
								{/each}
							</div>
						</div>
					</div>
				{/if}
			</div>
		</div>
	{:else}
		<div class="py-20 text-center">
			<p class="text-base-content/60">Item not found</p>
		</div>
	{/if}
</div>
