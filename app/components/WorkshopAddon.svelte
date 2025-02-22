<script>
	import { tippyFollow } from '../tippy.js';
	import { modals } from '../modals.js';
	import PreviewGMA from '../modals/PreviewGMA.svelte';
	import Dead from '../../public/img/dead.svg';
	import { Addons } from '../addons.js';
	import Loading from '../components/Loading.svelte';

	export let id;
	export let title;
	export let timeCreated;
	export let timeUpdated;
	export let score;
	export let tags;
	export let previewUrl;
	export let subscriptions;
	export let localFile;
	export let steamid64;
	export let searchTitle;

	export let dead;
	export let loading = false;

	export let isPreviewing = false;

	let stars = Math.round((score * 10) / 2);
	let starsPct = Math.round(((score * 100) + Number.EPSILON) * 100) / 100;

	const getMetadata = dead && !!localFile ? Addons.getGMAMetadata(localFile, id) : undefined;

	function click() {
		if (!localFile) return;
		$modals = [...$modals, {
			component: PreviewGMA,
			props: {
				workshopData: $$props
			}
		}];
	}
</script>

<div id="workshop-addon" class="ws-{id}" class:previewing={isPreviewing} data-ws={id}>
	<div id="card" on:click={ !isPreviewing ? click : null }>
		<div id="stats">
			<span id="subscriptions">
				<img src="/img/download.png" alt="Subscriptions"/>
				{subscriptions}
				&nbsp;
			</span>
			<img use:tippyFollow={starsPct + '%'} id="score" src="/img/{stars}-star.png" alt="{stars} Stars"/>
		</div>
		{#if dead || loading}
			<div id="preview" class="dead">
				{#if loading}
					<Loading size="2rem"/>
				{:else}
					<Dead/>
				{/if}
			</div>
			{#await getMetadata}
				<div id="title">{title}</div>
			{:then metadata}
				{#if metadata}
					<div id="title">{metadata.name}</div>
				{/if}
			{:catch fileName}
				<div id="title">{fileName}</div>
			{/await}
		{:else}
			<img id="preview" src={previewUrl} alt="Preview"/>
			<div id="title">{title}</div>
		{/if}
	</div>
</div>

<style>
	#workshop-addon {
		display: flex;
		flex-direction: column;
		height: 100%;
	}
	#workshop-addon #card {
		padding: .8rem;
		transition: background-color .1s, box-shadow .1s;
	}
	#workshop-addon:not(.previewing) {
		cursor: pointer;
	}
	#workshop-addon:not(.previewing):hover #card {
		background-color: rgba(45, 45, 45, 1);
		box-shadow: 0px 0px 4px rgb(0 0 0 / 40%);
	}
	#workshop-addon #title {
		margin-top: .8rem;
		flex: 1;
		text-align: center;
	}
	#workshop-addon #stats {
		display: flex;
		flex-direction: row;
		text-align: center;
		align-items: center;
	}
	#workshop-addon #stats * {
		vertical-align: middle;
	}
	#workshop-addon #subscriptions {
		flex: 1;
		text-align: left;
	}
	#workshop-addon #subscriptions img {
		margin-right: .1rem;
	}
	#workshop-addon #preview {
		width: 100%;
		flex: 0;
		margin-top: .8rem;
		box-shadow: 0 0 2px 1px rgba(0, 0, 0, .5);
	}
	#workshop-addon #preview.dead {
		position: relative;
		background-color: #0c0c0c;
	}
	#workshop-addon #preview.dead::after {
		content: "";
		display: block;
		padding-bottom: 100%;
	}
	#workshop-addon #preview.dead :global(svg) {
		position: absolute;
		margin: auto;
		left: 0;
		top: 0;
		right: 0;
		bottom: 0;
		color: #212121;
	}
	#workshop-addon :global(.highlight) {
		background-color: rgba(255, 255, 0, .5);
		border-radius: 4px;
		box-shadow: 0 0 2px rgba(255, 255, 0, .5);
		color: black;
		padding: 2px;
		padding-left: 4px;
		padding-right: 4px;
		margin: -4px;
	}
</style>