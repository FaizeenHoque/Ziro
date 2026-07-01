<script lang="ts">
	import { onMount } from 'svelte';

	let ms = $state(0);
	let bootDone = $state(false);
	let showCursor = $state(true);

	onMount(() => {
		const target = 9;
		const duration = 700;
		const start = performance.now();

		function tick(now: number) {
			const t = Math.min(1, (now - start) / duration);
			ms = Math.floor(t * target);
			if (t < 1) {
				requestAnimationFrame(tick);
			} else {
				ms = target;
				bootDone = true;
			}
		}
		requestAnimationFrame(tick);

		const blink = setInterval(() => (showCursor = !showCursor), 530);
		return () => clearInterval(blink);
	});
</script>

<section class="relative pt-36 sm:pt-44 pb-20 px-6">
	<div class="mx-auto max-w-5xl">
		<div class="max-w-2xl">
			<p class="font-mono text-[13px] text-[#1F7A4C] mb-5">terminal editor · linux · rust</p>
			<h1
				class="font-mono text-[38px] sm:text-[54px] leading-[1.08] tracking-tight text-[#14140F]"
			>
				It opens before<br />you finish blinking.
			</h1>
			<p class="mt-6 text-[17px] leading-relaxed text-[#5B5C52] max-w-lg">
				Ziro is a featherlight terminal text editor for Linux. No Electron. No Chromium.
				No unnecessary layers between you and your keystrokes.
			</p>

			<div class="mt-8 flex flex-wrap items-center gap-5">
				<a
					href="#install"
					class="rounded-full bg-[#14140F] px-6 py-3 font-mono text-[13px] text-[#FCFCFA] hover:bg-[#1F7A4C] transition-colors"
				>
					Install Ziro
				</a>
				<a
					href="https://github.com/FaizeenHoque/ziro"
					target="_blank"
					rel="noreferrer"
					class="font-mono text-[13px] text-[#5B5C52] hover:text-[#14140F] underline underline-offset-4 decoration-[#E5E3D8] transition-colors"
				>
					Read the source →
				</a>
			</div>
		</div>

		<!-- signature element: dark terminal window, boot-time counter -->
		<div
			class="mt-16 rounded-xl border border-[#E5E3D8] bg-[#14140F] overflow-hidden max-w-3xl shadow-[0_20px_50px_-25px_rgba(20,20,15,0.4)]"
		>
			<div class="flex items-center justify-between px-4 h-9 border-b border-white/10">
				<span class="font-mono text-[12px] text-white/40">ziro — ~/projects/ziro/src/main.rs</span>
				<span class="font-mono text-[12px] text-white/40">
					{#if bootDone}opened in {ms}ms{:else}opening…{/if}
				</span>
			</div>
			<div class="p-5 font-mono text-[13px] leading-relaxed overflow-x-auto">
				<div class="flex"><span class="w-8 text-white/25 select-none">1</span><span><span class="text-[#1F7A4C]">use</span> <span class="text-white/70">ziro::editor::Editor;</span></span></div>
				<div class="flex"><span class="w-8 text-white/25 select-none">2</span><span><span class="text-[#1F7A4C]">use</span> <span class="text-white/70">std::io;</span></span></div>
				<div class="flex mt-3"><span class="w-8 text-white/25 select-none">3</span><span><span class="text-[#7DA98F]">fn</span> <span class="text-white/90">main</span><span class="text-white/50">() -&gt; io::Result&lt;()&gt; {'{'}</span></span></div>
				<div class="flex"><span class="w-8 text-white/25 select-none">4</span><span class="pl-4"><span class="text-[#7DA98F]">let</span> <span class="text-white/50">mut</span> <span class="text-white/90">editor</span> <span class="text-white/50">=</span> <span class="text-white/70">Editor::new()?;</span></span></div>
				<div class="flex"><span class="w-8 text-white/25 select-none">5</span><span class="pl-4"><span class="text-white/90">editor</span><span class="text-white/50">.run(){showCursor ? '▏' : ''}</span></span></div>
				<div class="flex"><span class="w-8 text-white/25 select-none">6</span><span class="text-white/50">{'}'}</span></div>
			</div>
			<div class="flex items-center gap-6 px-4 h-8 bg-[#1B1B14] font-mono text-[11px] text-white/40">
				<span>NORMAL</span>
				<span>main.rs</span>
				<span class="ml-auto">ziro 0.1.5</span>
			</div>
		</div>
	</div>
</section>