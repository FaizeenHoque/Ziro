<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import Lenis from '@studio-freight/lenis';
	import Nav from '$lib/components/Nav.svelte';
	import Hero from '$lib/components/Hero.svelte';
	import Why from '$lib/components/Why.svelte';
	import Goals from '$lib/components/Goals.svelte';
	import Stack from '$lib/components/Stack.svelte';
	import Install from '$lib/components/Install.svelte';
	import Footer from '$lib/components/Footer.svelte';

	let lenis: Lenis | undefined;
	let animationFrameId: number | undefined;

	onMount(() => {
		const lenisInstance = new Lenis({
			duration: 1.5,
			easing: (t) => Math.min(1, 1.001 - Math.pow(2, -10 * t)),
			orientation: 'vertical',
			gestureOrientation: 'vertical',
			smoothWheel: true
		});
		lenis = lenisInstance;

		function raf(time: number) {
			lenisInstance.raf(time);
			animationFrameId = requestAnimationFrame(raf);
		}
		animationFrameId = requestAnimationFrame(raf);
	});

	onDestroy(() => {
		if (animationFrameId) cancelAnimationFrame(animationFrameId);
		if (lenis) lenis.destroy();
	});
</script>

<svelte:head>
	<title>Ziro — a featherlight terminal editor for Linux</title>
	<meta
		name="description"
		content="Ziro is a minimal, modal terminal text editor for Linux, built in Rust. No Electron. No Chromium. No unnecessary layers."
	/>
</svelte:head>

<div class="bg-[#FCFCFA] min-h-screen font-sans text-[#14140F]">
	<Nav />
	<main>
		<Hero />
		<Why />
		<Goals />
		<Stack />
		<Install />
	</main>
	<Footer />
</div>