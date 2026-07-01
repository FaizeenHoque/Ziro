<script lang="ts">
	import { reveal } from '$lib/actions/reveal';

	const rows = [
		{ name: 'VS Code', cost: 'Ships an entire browser just to edit text.' },
		{ name: 'Neovim', cost: 'Unlimited power, after a configuration ritual.' },
		{ name: 'Zed', cost: 'Promising, but still finding its footing on Linux.' },
		{ name: 'Sublime Text', cost: 'Costs money.' }
	];
</script>

<section id="why" class="px-6 py-24 border-t border-[#E5E3D8]" use:reveal>
	<div class="mx-auto max-w-5xl">
		<p class="font-mono text-[13px] text-[#1F7A4C] mb-3">why</p>
		<h2 class="font-mono text-[28px] sm:text-[34px] tracking-tight text-[#14140F] max-w-xl">
			Every modern editor makes you pay somewhere.
		</h2>

		<div class="mt-12 max-w-2xl divide-y divide-[#E5E3D8]">
			{#each rows as row (row.name)}
				<div class="flex flex-col sm:flex-row sm:items-baseline gap-1 sm:gap-8 py-4">
					<span class="font-mono text-[14px] text-[#5B5C52] sm:w-32 shrink-0">{row.name}</span>
					<span class="text-[15px] text-[#14140F]">{row.cost}</span>
				</div>
			{/each}
			<div class="flex flex-col sm:flex-row sm:items-baseline gap-1 sm:gap-8 py-4">
				<span class="font-mono text-[14px] text-[#1F7A4C] sm:w-32 shrink-0">Ziro</span>
				<span class="text-[15px] text-[#14140F]">None of the above.</span>
			</div>
		</div>
	</div>
</section>