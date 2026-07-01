<script lang="ts">
	import { onMount } from 'svelte';
	import { base } from '$app/paths';
	import ziroMark from '$lib/assets/ziro_mark.svg';

	let scrolled = $state(false);

	onMount(() => {
		function onScroll() {
			scrolled = window.scrollY > 8;
		}
		onScroll();
		window.addEventListener('scroll', onScroll, { passive: true });
		return () => window.removeEventListener('scroll', onScroll);
	});

	const links = [
		{ href: `${base}/#why`, label: 'Why' },
		{ href: `${base}/#goals`, label: 'Goals' },
		{ href: `${base}/#stack`, label: 'Stack' },
		{ href: `${base}/#install`, label: 'Install' },
		{ href: `${base}/docs`, label: 'Docs' }
	];
</script>

<header
	class="fixed top-0 inset-x-0 z-50 transition-all duration-300"
	class:border-b={scrolled}
	class:border-[#E5E3D8]={scrolled}
	class:bg-[#FCFCFA]={scrolled}
>
	<nav class="mx-auto max-w-5xl flex items-center justify-between px-6 h-16">
		<a href={`${base}/`} class="flex items-center gap-2.5 shrink-0">
			<img src={ziroMark} alt="Ziro" class="h-30 w-30 rounded-[6px]" />
			<!-- <span class="font-mono text-[15px] tracking-tight text-[#14140F]">ziro</span> -->
		</a>

		<div class="hidden md:flex items-center gap-8">
			{#each links as link (link.href)}
				<a
					href={link.href}
					class="font-mono text-[13px] text-[#5B5C52] hover:text-[#14140F] transition-colors"
				>
					{link.label}
				</a>
			{/each}
		</div>

		<a
			href="https://github.com/FaizeenHoque/ziro"
			target="_blank"
			rel="noreferrer"
			class="inline-flex items-center gap-2 rounded-full border border-[#14140F] px-4 py-1.5 font-mono text-[13px] text-[#14140F] hover:bg-[#14140F] hover:text-[#FCFCFA] transition-colors"
		>
			GitHub
		</a>
	</nav>
</header>