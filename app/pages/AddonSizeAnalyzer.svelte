<script>
	import { afterUpdate, onDestroy, onMount } from 'svelte';
	import { _ } from 'svelte-i18n';
	import { promisified, invoke } from 'tauri/api/tauri';
	import Loading from '../components/Loading.svelte';
	import Dead from '../../public/img/dead.svg';
	import { Transaction } from '../transactions.js';
	import filesize from 'filesize';
	import tippy from 'tippy.js';
	import { modals } from '../modals.js';
	import PreviewGMA from '../modals/PreviewGMA.svelte';
	import { writable } from 'svelte/store';

	const tagCache = {}; let tagCacheId = 0;
	function getTagId(tag) {
		if (!(tag in tagCache)) {
			const id = tagCacheId++;
			tagCache[tag] = id;
			tagCache[id] = tag;
		}
		return tagCache[tag];
	}
	function lookupTagName(tagId) {
		return tagCache[tagId];
	}

	const tagColors = {};
	function getTagColor(tag) {
		if (!tag) return null;

		if (!(tag in tagColors)) {
			const elem = document.createElement('span');
			elem.classList.add('tag', tag);
			document.body.appendChild(elem);

			const color = window.getComputedStyle(elem, 'before')?.getPropertyValue('background-color');
			tagColors[tag] = !!color ? color : null;

			elem.remove();
		}

		return tagColors[tag];
	}

	let container;

	let addonsCanvas; let addonsCtx;
	let tagsCanvas; let tagsCtx;
	let bgCanvas; let bgCtx;

	let loading = new Promise(() => {});

	let transaction;
	let progressMsg = writable(null);

	let treemapBounds = [];
	let treemap;

	const DeadSVG = new Image();
	DeadSVG.src = '/img/dead-canvas.svg';

	function analyze() {
		transaction?.cancel();

		treemapBounds[0] = container.clientWidth;
		treemapBounds[1] = container.clientHeight;

		promisified({

			cmd: 'analyzeAddonSizes',
			w: treemapBounds[0],
			h: treemapBounds[1]

		}).then(transactionId => {
			transaction = new Transaction(transactionId);

			loading = new Promise((resolve, reject) => {
				transaction.listen(event => {
					if (event.finished) {
						transaction.progress = 100;
						transaction.setStatus('FS_ANALYZER_RENDERING');
						treemap = event.data;
						resolve();
					} else if (event.error != null) {
						reject([event.error, event.data]);
					}
					$progressMsg = transaction.status;
					transaction = transaction;
				});
			});
		});
	}

	let resizedTimeout;
	function resized() {
		if (treemapBounds[0] !== container.clientWidth || treemapBounds[1] !== container.clientHeight) {
			clearTimeout(resizedTimeout);
			resizedTimeout = setTimeout(analyze, 1000);
		}
	}

	let addonsKdTreePoints = [];
	let addonsKdTree;
	let addonsKdMaxDist = 0;

	const hangingLetters = ['g','j','p','q','y']; // stupid hack for annoying canvas text rendering

	async function findHoveredAddon(e, shouldSelect) {
		if (!addonsKdTree) return;

		const { offsetX, offsetY } = e;

		const nearest = addonsKdTree.nearest({ x: offsetX, y: offsetY }, addonsKdTreePoints.length, addonsKdMaxDist);
		if (nearest) {
			for (let i = 0; i < nearest.length; i++) {
				const addon = nearest[i][0];
				if (offsetX >= addon.left && offsetX <= addon.right && offsetY >= addon.top && offsetY <= addon.bottom) {
					if (shouldSelect) selectHoveredSquare(addon);
					return addon;
				}
			}
		}

		if (shouldSelect) selectHoveredSquare();
	}

	const tagLabels = {};

	const canvasRotation = -90 * Math.PI / 180;
	function processTreemap(treemap, offsetX, offsetY, padding, tagId) {
		for (let i = 0; i < treemap.length; i++) {
			const square = treemap[i];
			const [data, tag] = square.data;

			const x = square.x + offsetX + (padding || 0);
			const y = square.y + offsetY + (padding || 0);
			
			const halfW = square.w / 2;
			const halfH = square.h / 2;

			const centerX = x + halfW;
			const centerY = y + halfH;

			if (tag) {
				
				let padding = Math.ceil((Math.min(square.w, square.h) * 0.05)) / 2;

				const tagColor = getTagColor(tag);
				bgCtx.fillStyle = tagColor ?? '#0c0c0c';
				bgCtx.fillRect(x, y, square.w, square.h);

				let hanging = false;
				for (let i = 0; i < hangingLetters.length; i++) {
					if (tag.indexOf(hangingLetters[i]) !== -1) {
						hanging = true;
						break;
					}
				}

				tagLabels[tag] = {
					x: centerX,
					y: centerY,
					w: square.w,
					h: square.h,
					padding,
					hanging,
				};
				drawTag(tag);
				
				if (Array.isArray(data)) {
					processTreemap(data, x, y, padding, getTagId(tag));
				}

				continue;

			} else {

				if (data.preview_url) {

					const image = new Image(square.w, square.h);
					image.onload = () => {
						addonsCtx.globalCompositeOperation = 'destination-over';
							addonsCtx.drawImage(image, x, y, square.w, square.h);
						addonsCtx.globalCompositeOperation = 'source-over';
					}
					image.src = data.preview_url;

				} else {

					addonsCtx.globalCompositeOperation = 'destination-over';
						const size = square.w * .25;
						const halfSize = size / 2;
						addonsCtx.drawImage(DeadSVG, centerX - halfSize, centerY - halfSize, size, size);

						addonsCtx.fillStyle = '#0c0c0c';
						addonsCtx.fillRect(x, y, square.w, square.h);
					addonsCtx.globalCompositeOperation = 'source-over';

				}

			}

			data.tagId = tagId;

			data.left = x;
			data.right = x + square.w;
			data.top = y;
			data.bottom = y + square.h;

			data.x = x;
			data.y = y;
			data.w = square.w;
			data.h = square.h;

			addonsKdMaxDist = Math.max(addonsKdMaxDist, (data.w ** 2) + (data.h ** 2));
			addonsKdTreePoints.push(data);
		}
	}

	function drawTag(tag) {
		let { w, h, x, y, padding, hanging } = tagLabels[tag];

		const aspectRatio = w / h;
		const verticalText = aspectRatio <= 0.33;

		const textBoundsWidth = Math.floor((w - padding) * .75);
		const textBoundsHeight = Math.floor((h - padding) * .75);
		let textSize = Math.floor(w);
		let metrics;
		while (textSize > 1) {
			tagsCtx.font = textSize-- + 'px sans-serif';
			metrics = tagsCtx.measureText(tag);
			const height = hanging ? ((metrics.fontBoundingBoxAscent ?? 0) + (metrics.fontBoundingBoxDescent ?? 0)) : ((metrics.actualBoundingBoxAscent ?? 0) + (metrics.actualBoundingBoxDescent ?? 0));

			if (
				metrics.width < (verticalText ? textBoundsHeight : textBoundsWidth) &&
				height < (verticalText ? textBoundsWidth : textBoundsHeight)
			) break;
		}

		if (verticalText) {
			tagsCtx.translate(x, y);
				tagsCtx.rotate(canvasRotation);
			tagsCtx.translate(-x, -y);
		}
		
		tagsCtx.font = textSize + 'px sans-serif';
		tagsCtx.fillStyle = '#fff';
		tagsCtx.strokeStyle = '#000';
		tagsCtx.lineWidth = Math.min(Math.max(Math.ceil(textSize * 0.2), 4), (14 / 1920) * window.outerWidth);
		tagsCtx.strokeText(tag, x, y);
		tagsCtx.fillText(tag, x, y);

		if (verticalText) {
			tagsCtx.setTransform(1, 0, 0, 1, 0, 0);
		}
	}

	let tagFiltering;
	function updateTagCanvas(filter) {
		if (tagFiltering === filter) return;
		tagFiltering = filter;

		tagsCtx.clearRect(0, 0, tagsCanvas.width, tagsCanvas.height);

		for (let tag in tagLabels) {
			if (tag !== filter)
				drawTag(tag);
		}
	}

	function updateCanvas() {
		if (!bgCanvas || !tagsCanvas || !treemap) return;
		addonsKdTree = null;
		addonsKdTreePoints = [];
		addonsKdMaxDist = 0;

		const scale = window.devicePixelRatio;
		const width = container.clientWidth * scale;
		const height = container.clientHeight * scale;

		bgCanvas.width = Math.floor(width);
		bgCanvas.height = Math.floor(height);

		tagsCanvas.width = Math.floor(width);
		tagsCanvas.height = Math.floor(height);

		addonsCanvas.width = Math.floor(width);
		addonsCanvas.height = Math.floor(height);

		bgCtx = bgCanvas.getContext('2d');
		bgCtx.scale(scale, scale);
		bgCtx.clearRect(0, 0, bgCanvas.width, bgCanvas.height);

		tagsCtx = tagsCanvas.getContext('2d');
		tagsCtx.scale(scale, scale);
		tagsCtx.clearRect(0, 0, tagsCanvas.width, tagsCanvas.height);
		tagsCtx.textAlign = 'center';
		tagsCtx.textBaseline = 'middle';
		tagsCtx.lineJoin = 'round';

		addonsCtx = addonsCanvas.getContext('2d');
		addonsCtx.scale(scale, scale);
		addonsCtx.clearRect(0, 0, addonsCanvas.width, addonsCanvas.height);

		processTreemap(treemap, 0, 0);

		treemap = null;

		addonsKdTree = new kdTree(addonsKdTreePoints, (a, b) => {
			return ((a.x - b.x) ** 2) + ((a.y - b.y) ** 2);
		}, ['x', 'y']);
	}

	onMount(analyze);
	afterUpdate(updateCanvas);
	onDestroy(() => {
		clearTimeout(resizedTimeout);
		transaction?.cancel();
		invoke({ cmd: 'freeAddonSizeAnalyzer' });
	});

	let popper;
	let popperContent;
	let popperName;
	let popperType;
	let popperSize;

	onMount(() => {
		tippy(popper, {
			allowHTML: true,
			arrow: true,
			interactive: false
		});
	});

	function selectHoveredSquare(addon) {
		if (!addon) {

			updateTagCanvas();

			popper.style.width = '0';
			popper.style.height = '0';
			popper._tippy.hide();

		} else {

			popperName.textContent = addon.gma.name ?? addon.gma.extracted_name;
			popperSize.textContent = filesize(Number(addon.gma.size));
			
			const tagName = lookupTagName(addon.tagId);
			updateTagCanvas(tagName);
			popperType.setAttribute('class', 'tag ' + tagName);
			popperType.textContent = tagName;

			popper.style.boxShadow = `inset 0 0 0 .25rem ${getTagColor(lookupTagName(addon.tagId)) ?? '#0c0c0c'}`;
			popper.style.left = addon.left + 'px';
			popper.style.top = addon.top + 'px';
			popper.style.width = addon.w + 'px';
			popper.style.height = addon.h + 'px';

			popper._tippy.setContent(popperContent.innerHTML);
			popper._tippy.show();

		}
		
		return addon;
	}

	async function openHoveredAddon(e) {
		let addon = await findHoveredAddon(e, false);
		selectHoveredSquare();
		if (addon) {
			$modals = [...$modals, {
				component: PreviewGMA,
				props: {
					gmaData: addon.gma
				}
			}];
		}
	}

	let timeouts = [];
	onDestroy(() => timeouts.map(clearTimeout));

	let progressLog;
	progressMsg.subscribe(msg => {
		if (!!!msg || !progressLog) return;

		if (progressLog.firstElementChild) {
			const elem = progressLog.firstElementChild;
			elem.classList.add('stale');
			timeouts.push(setTimeout(elem => {
				if (!elem) return;
				elem.remove();
			}, 2500, elem));
		}

		const elem = document.createElement('div');
		elem.classList.add('log');
		elem.textContent = $_(msg);
		progressLog.prepend(elem);
	});
	onMount(() => $progressMsg = $progressMsg);
</script>

<svelte:window on:resize={resized}/>

<div id="popper-content" bind:this={popperContent}>
	<table class="popper-content">
		<tbody>
			<tr>
				<th>{$_('name')}</th>
				<td bind:this={popperName}></td>
			</tr>
			<tr>
				<th>{$_('addon_type')}</th>
				<td><div bind:this={popperType}></div></td>
			</tr>
			<tr>
				<th>{$_('size')}</th>
				<td bind:this={popperSize}></td>
			</tr>
		</tbody>
	</table>
</div>

<main bind:this={container}>
	<script src="/js/lib/k-d-tree.min.js"></script>
	<div id="popper" bind:this={popper}></div>

	{#await loading}
		<div id="loading">
			<Loading size="2rem"/>
			{#if transaction}
				<div id="progress">
					{transaction.progress.toFixed(2)}%
					<div id="bar" style="width:calc({transaction.progress}% - .6rem)"></div>
				</div>
				<div id="progress-log" bind:this={progressLog}></div>
			{:else}
				<div id="progress">0.00%</div>
			{/if}
		</div>
	{:then}
		<canvas bind:this={bgCanvas}></canvas>
		<canvas bind:this={addonsCanvas}></canvas>
		<canvas bind:this={tagsCanvas}
			on:mousemove={e => findHoveredAddon(e, true)}
			on:mouseout={() => selectHoveredSquare()}
			on:click={openHoveredAddon}
		></canvas>
	{:catch}
		<Dead/>
	{/await}
</main>

<style>
	main {
		background-color: #1a1a1a;
		border-radius: .3rem;
		box-shadow: 0 0 0 #000 inset;

		width: 100%;
		height: calc(100% - 1.5rem);
		max-height: calc(100% - 1.5rem);
		min-height: calc(100% - 1.5rem);

		display: grid;
		grid-template-columns: 1fr;
		grid-template-rows: 1fr;

		position: relative;
	}
	main > canvas {
		grid-row: 1;
		grid-column: 1;
		width: 100%;
		height: 100%;
		cursor: pointer;
	}

	#popper {
		position: absolute;
		width: 0;
		height: 0;
		pointer-events: none;

		transition: width .1s, height .1s, top .1s, left .1s, right .1s, bottom .1s;
	}
	#popper-content {
		display: none;
	}
	:global(.popper-content) {
		text-align: left;
	}

	#loading {
		grid-row: 1;
		grid-column: 1;
		width: 100%;
		height: 100%;
		display: flex;
		flex-direction: column;
		justify-content: center;
		align-items: center;
	}
	:global(#loading > .loading) { margin: 0 !important }
	#progress {
		display: inline-block;
		position: relative;
		width: 15%;
		background-color: rgba(0,0,0,.4);
		z-index: 1;
		color: #fff;
		text-align: center;
		text-shadow: 0px 1px 0px rgba(0, 0, 0, 0.6);
		padding: .5rem;

		margin-top: .8rem;
		margin-bottom: .8rem;
	}
	#progress > #bar {
		position: absolute;
		z-index: -1;
		height: calc(100% - .6rem);
		background-color: #006cc7;
		top: .3rem;
		left: .3rem;
	}

	#progress-log {
		text-shadow: 0px 1px 0px rgba(0, 0, 0, 0.6);
		text-align: center;
		max-height: 1rem;
	}
	#progress-log :global(.log:not(:last-child)) {
		margin-bottom: .5rem;
	}
	#progress-log :global(.log.stale) {
		animation: stale 2.5s;
		animation-fill-mode: forwards;
	}

	@keyframes stale {
		from { opacity: 1 }
		to { opacity: 0 }
	}
</style>