<!-- TODO remove svg img from assets -->
<script>
	export let size = '1rem';
	export let inline = false;
</script>

<svg class="dead" class:inline={inline} xmlns="http://www.w3.org/2000/svg" width={size} height={size} viewBox="0 0 48 48" fill="none"><path stroke="currentColor" stroke-width="3" stroke-linecap="square" d="M1.5 8.5v34h45v-28m-3-3h-10v-3m-3-3h-10m15 6h-18v-3m-3-3h-10"/><path stroke="currentColor" stroke-width="2" stroke-linecap="square" d="M12 35h2m2-2h12m2 2h3m2 2h3M11 21h0m0 4h0m4 0h0m0-4h0m-2 2h0m20-2h0m0 4h0m4 0h0m0-4h0m-2 2h0"/></svg>

<style>
	.dead:not(.inline) {
		display: block;
		margin: auto;
	}
</style>