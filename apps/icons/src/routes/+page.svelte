<script lang="ts">
	import { onMount } from 'svelte';

	const CANVAS_SIZE = 512;

	const TAURI_SIZES = [
		{ name: '32x32.png', size: 32 },
		{ name: '128x128.png', size: 128 },
		{ name: '128x128@2x.png', size: 256 },
		{ name: 'icon.png', size: 512 },
		{ name: 'Square30x30Logo.png', size: 30 },
		{ name: 'Square44x44Logo.png', size: 44 },
		{ name: 'Square71x71Logo.png', size: 71 },
		{ name: 'Square89x89Logo.png', size: 89 },
		{ name: 'Square107x107Logo.png', size: 107 },
		{ name: 'Square142x142Logo.png', size: 142 },
		{ name: 'Square150x150Logo.png', size: 150 },
		{ name: 'Square284x284Logo.png', size: 284 },
		{ name: 'Square310x310Logo.png', size: 310 },
		{ name: 'StoreLogo.png', size: 50 },
		{ name: 'source-1024x1024.png', size: 1024 }
	];

	let canvas: HTMLCanvasElement;
	let text = $state('M');
	let fontSize = $state(320);
	let fontFamily = $state('sans-serif');
	let bgColor = $state('#1e293b');
	let textColor = $state('#ffffff');
	let borderRadius = $state(80);
	let offsetX = $state(0);
	let offsetY = $state(0);
	let bold = $state(true);

	const fontOptions = [
		'sans-serif',
		'serif',
		'monospace',
		'system-ui',
		'Georgia',
		'Courier New',
		'Arial',
		'Helvetica',
		'Times New Roman'
	];

	function drawIcon(ctx: CanvasRenderingContext2D, size: number) {
		ctx.clearRect(0, 0, size, size);

		// Background with rounded corners
		const r = (borderRadius / CANVAS_SIZE) * size;
		ctx.beginPath();
		ctx.moveTo(r, 0);
		ctx.lineTo(size - r, 0);
		ctx.quadraticCurveTo(size, 0, size, r);
		ctx.lineTo(size, size - r);
		ctx.quadraticCurveTo(size, size, size - r, size);
		ctx.lineTo(r, size);
		ctx.quadraticCurveTo(0, size, 0, size - r);
		ctx.lineTo(0, r);
		ctx.quadraticCurveTo(0, 0, r, 0);
		ctx.closePath();
		ctx.fillStyle = bgColor;
		ctx.fill();

		// Clip to rounded rect for text
		ctx.save();
		ctx.clip();

		// Text
		const scaledFontSize = (fontSize / CANVAS_SIZE) * size;
		const weight = bold ? 'bold' : 'normal';
		ctx.font = `${weight} ${scaledFontSize}px ${fontFamily}`;
		ctx.fillStyle = textColor;
		ctx.textAlign = 'center';
		ctx.textBaseline = 'middle';
		const ox = (offsetX / CANVAS_SIZE) * size;
		const oy = (offsetY / CANVAS_SIZE) * size;
		ctx.fillText(text, size / 2 + ox, size / 2 + oy);

		ctx.restore();
	}

	function redraw() {
		if (!canvas) return;
		const ctx = canvas.getContext('2d');
		if (!ctx) return;
		drawIcon(ctx, CANVAS_SIZE);
	}

	$effect(() => {
		// Track reactive dependencies
		void [text, fontSize, fontFamily, bgColor, textColor, borderRadius, offsetX, offsetY, bold];
		redraw();
	});

	onMount(() => {
		redraw();
	});

	function exportAll() {
		for (const { name, size } of TAURI_SIZES) {
			const offscreen = document.createElement('canvas');
			offscreen.width = size;
			offscreen.height = size;
			const ctx = offscreen.getContext('2d');
			if (!ctx) continue;
			drawIcon(ctx, size);

			offscreen.toBlob((blob) => {
				if (!blob) return;
				const url = URL.createObjectURL(blob);
				const a = document.createElement('a');
				a.href = url;
				a.download = name;
				a.click();
				URL.revokeObjectURL(url);
			});
		}
	}

	function exportSingle(name: string, size: number) {
		const offscreen = document.createElement('canvas');
		offscreen.width = size;
		offscreen.height = size;
		const ctx = offscreen.getContext('2d');
		if (!ctx) return;
		drawIcon(ctx, size);

		offscreen.toBlob((blob) => {
			if (!blob) return;
			const url = URL.createObjectURL(blob);
			const a = document.createElement('a');
			a.href = url;
			a.download = name;
			a.click();
			URL.revokeObjectURL(url);
		});
	}
</script>

<div class="mx-auto flex max-w-6xl flex-col gap-6 p-6">
	<h1 class="text-2xl font-bold">Tauri Icon Maker</h1>

	<div class="flex flex-col gap-6 lg:flex-row">
		<!-- Canvas preview -->
		<div class="flex flex-col items-center gap-4">
			<div class="rounded-lg bg-base-300 p-4">
				<canvas
					bind:this={canvas}
					width={CANVAS_SIZE}
					height={CANVAS_SIZE}
					class="h-64 w-64 rounded"
					style="image-rendering: auto;"
				></canvas>
			</div>
			<p class="text-sm text-base-content/50">Preview ({CANVAS_SIZE}×{CANVAS_SIZE})</p>
		</div>

		<!-- Controls -->
		<div class="flex flex-1 flex-col gap-4">
			<div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
				<label class="form-control w-full">
					<div class="label"><span class="label-text">Text</span></div>
					<input
						type="text"
						bind:value={text}
						class="input-bordered input w-full"
						placeholder="Icon text"
					/>
				</label>

				<label class="form-control w-full">
					<div class="label"><span class="label-text">Font</span></div>
					<select bind:value={fontFamily} class="select-bordered select w-full">
						{#each fontOptions as font}
							<option value={font}>{font}</option>
						{/each}
					</select>
				</label>

				<label class="form-control w-full">
					<div class="label">
						<span class="label-text">Font Size: {fontSize}px</span>
					</div>
					<input
						type="range"
						min="32"
						max="500"
						bind:value={fontSize}
						class="range range-primary"
					/>
				</label>

				<label class="form-control w-full">
					<div class="label">
						<span class="label-text">Border Radius: {borderRadius}px</span>
					</div>
					<input
						type="range"
						min="0"
						max="256"
						bind:value={borderRadius}
						class="range range-primary"
					/>
				</label>

				<label class="form-control w-full">
					<div class="label">
						<span class="label-text">Offset X: {offsetX}px</span>
					</div>
					<input
						type="range"
						min="-200"
						max="200"
						bind:value={offsetX}
						class="range range-secondary"
					/>
				</label>

				<label class="form-control w-full">
					<div class="label">
						<span class="label-text">Offset Y: {offsetY}px</span>
					</div>
					<input
						type="range"
						min="-200"
						max="200"
						bind:value={offsetY}
						class="range range-secondary"
					/>
				</label>

				<label class="form-control w-full">
					<div class="label"><span class="label-text">Background</span></div>
					<input type="color" bind:value={bgColor} class="input-bordered input h-10 w-full p-1" />
				</label>

				<label class="form-control w-full">
					<div class="label"><span class="label-text">Text Color</span></div>
					<input type="color" bind:value={textColor} class="input-bordered input h-10 w-full p-1" />
				</label>
			</div>

			<label class="label cursor-pointer justify-start gap-3">
				<input type="checkbox" bind:checked={bold} class="checkbox checkbox-primary" />
				<span class="label-text">Bold</span>
			</label>

			<button class="btn mt-2 btn-primary" onclick={exportAll}>Export All Tauri Icon Sizes</button>
		</div>
	</div>

	<!-- Export sizes table -->
	<div class="overflow-x-auto">
		<h2 class="mb-3 text-lg font-semibold">Export Sizes</h2>
		<table class="table table-sm">
			<thead>
				<tr>
					<th>Filename</th>
					<th>Size</th>
					<th>Action</th>
				</tr>
			</thead>
			<tbody>
				{#each TAURI_SIZES as { name, size }}
					<tr>
						<td class="font-mono text-sm">{name}</td>
						<td>{size}×{size}</td>
						<td>
							<button class="btn btn-ghost btn-xs" onclick={() => exportSingle(name, size)}>
								Download
							</button>
						</td>
					</tr>
				{/each}
			</tbody>
		</table>
	</div>
</div>
