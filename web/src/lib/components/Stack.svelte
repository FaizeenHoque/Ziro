<script lang="ts">
	import { reveal } from '$lib/actions/reveal';

	const stack = [
		{ layer: 'Language', tech: 'Rust' },
		{ layer: 'UI', tech: 'ratatui' },
		{ layer: 'Text buffer', tech: 'ropey' },
		{ layer: 'Syntax', tech: 'tree-sitter' },
		{ layer: 'Config', tech: 'toml + serde' },
		{ layer: 'Async', tech: 'tokio' }
	];

	const done = [
		'Project scaffold',
		'Terminal interface',
		'Editor UI',
		'File open/save',
		'Syntax highlighting',
		'LSP integration'
	];

	const pending = ['Config system', 'Plugin system'];
</script>

<section id="stack" class="px-6 py-24 border-t border-[#E5E3D8]" use:reveal>
	<div class="mx-auto max-w-5xl grid md:grid-cols-2 gap-16">
		<div>
			<p class="font-mono text-[13px] text-[#1F7A4C] mb-3">stack</p>
			<h2 class="font-mono text-[24px] tracking-tight text-[#14140F] mb-8">
				Built with a small, boring toolset.
			</h2>
			<div class="divide-y divide-[#E5E3D8] border-t border-b border-[#E5E3D8]">
				{#each stack as row (row.layer)}
					<div class="flex items-center justify-between py-3">
						<span class="text-[14px] text-[#5B5C52]">{row.layer}</span>
						<span class="font-mono text-[13px] text-[#14140F]">{row.tech}</span>
					</div>
				{/each}
			</div>
		</div>

		<div>
			<p class="font-mono text-[13px] text-[#1F7A4C] mb-3">status</p>
			<h2 class="font-mono text-[24px] tracking-tight text-[#14140F] mb-2">
				Early development. Nothing is stable yet.
			</h2>
			<p class="text-[14px] text-[#5B5C52] mb-8">
				Don't reach for Ziro as your daily driver just yet — this is the real, current build state.
			</p>

			<div class="space-y-2.5">
				{#each done as item (item)}
					<div class="flex items-center gap-3">
						<span
							class="flex items-center justify-center w-4 h-4 rounded-[3px] bg-[#1F7A4C] text-[#FCFCFA] text-[10px] shrink-0"
							>✓</span
						>
						<span class="text-[14px] text-[#14140F]">{item}</span>
					</div>
				{/each}
				{#each pending as item (item)}
					<div class="flex items-center gap-3">
						<span class="w-4 h-4 rounded-[3px] border border-[#E5E3D8] shrink-0"></span>
						<span class="text-[14px] text-[#5B5C52]">{item}</span>
					</div>
				{/each}
			</div>
		</div>
	</div>
</section>
