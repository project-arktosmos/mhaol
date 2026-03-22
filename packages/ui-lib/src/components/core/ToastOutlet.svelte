<script lang="ts">
	import classNames from 'classnames';
	import { toastService } from 'ui-lib/services/toast.service';
	import type { ToastLevel } from 'ui-lib/types/toast.type';

	const alertClass: Record<ToastLevel, string> = {
		info: 'alert-info',
		success: 'alert-success',
		warning: 'alert-warning',
		error: 'alert-error'
	};
</script>

<div class="toast toast-end toast-top z-50">
	{#each $toastService as toast (toast.id)}
		<div class={classNames('alert shadow-lg', alertClass[toast.level])}>
			<div class="flex flex-col gap-2">
				<span>{toast.message}</span>
				{#if toast.actions?.length}
					<div class="flex gap-2">
						{#each toast.actions as action}
							<button
								class="btn btn-sm"
								onclick={() => {
									action.onclick();
									toastService.remove(toast.id);
								}}
							>
								{action.label}
							</button>
						{/each}
					</div>
				{/if}
			</div>
			<button class="btn btn-ghost btn-xs" onclick={() => toastService.remove(toast.id)}>
				✕
			</button>
		</div>
	{/each}
</div>
