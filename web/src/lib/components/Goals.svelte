<script lang="ts">
	import { reveal } from '$lib/actions/reveal';

	const goals = [
		{ title: 'Fast cold launch', body: 'Startup time you can feel, not one you have to estimate.' },
		{ title: 'Native terminal experience', body: 'Runs anywhere your shell does, no GUI runtime required.' },
		{ title: 'Rope-based text engine', body: 'Edits at any file size without copying the whole buffer.' },
		{ title: 'Tree-sitter syntax highlighting', body: 'Incremental parsing that stays correct while you type.' },
		{ title: 'LSP support', body: 'Autocomplete, go-to-definition, diagnostics — the full deal.' },
		{ title: 'Zero config to start', body: 'Sane defaults. Customize when you want to, not before you can use it.' },
		{ title: 'Minimal resource usage', body: 'A text editor should not need half your RAM.' }
	];
</script>

<section id="goals" class="px-6 py-24 border-t border-[#E5E3D8]" use:reveal>
	<div class="mx-auto max-w-5xl">
		<p class="font-mono text-[13px] text-[#1F7A4C] mb-3">goals</p>
		<h2 class="font-mono text-[28px] sm:text-[34px] tracking-tight text-[#14140F] max-w-xl">
			What Ziro is built to get right.
		</h2>

		<div class="mt-12 grid sm:grid-cols-2 gap-x-12 gap-y-8 max-w-3xl">
			{#each goals as goal (goal.title)}
				<div class="border-l border-[#E5E3D8] pl-5">
					<h3 class="font-mono text-[14px] text-[#14140F] mb-1.5">{goal.title}</h3>
					<p class="text-[14px] leading-relaxed text-[#5B5C52]">{goal.body}</p>
				</div>
			{/each}
		</div>
	</div>
</section>