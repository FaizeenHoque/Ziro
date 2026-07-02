<script lang="ts">
	import { reveal } from '$lib/actions/reveal';

	const commands = [
		{
			label: 'Stable',
			note: 'Prebuilt binary. No Rust required.',
			cmd: 'curl -sSL https://ziro.faizeenhoque.dev/install.sh | bash'
		},
		{
			label: 'Rolling',
			note: 'Builds from latest main. Requires Cargo.',
			cmd: 'curl -sSL https://ziro.faizeenhoque.dev/install-rolling.sh | bash'
		}
	];

	let copied = $state<string | null>(null);

	async function copy(cmd: string) {
		try {
			await navigator.clipboard.writeText(cmd);
			copied = cmd;
			setTimeout(() => {
				if (copied === cmd) copied = null;
			}, 1800);
		} catch {
			// clipboard unavailable, fail silently
		}
	}
</script>

<section id="install" class="px-6 py-24 border-t border-[#E5E3D8]" use:reveal>
	<div class="mx-auto max-w-5xl">
		<p class="font-mono text-[13px] text-[#1F7A4C] mb-3">install</p>
		<h2 class="font-mono text-[28px] sm:text-[34px] tracking-tight text-[#14140F] max-w-xl mb-4">
			Two commands. Pick one.
		</h2>
		<p class="text-[14px] text-[#5B5C52] mb-10 max-w-lg">
			Both scripts need <code class="font-mono text-[13px] text-[#14140F]">sudo</code> to move the
			binary to <code class="font-mono text-[13px] text-[#14140F]">/usr/local/bin</code>. Read the
			script first if you're cautious about piping curl to bash.
		</p>

		<div class="space-y-4 max-w-2xl">
			{#each commands as c (c.label)}
				<div>
					<div class="flex items-baseline justify-between mb-2">
						<span class="font-mono text-[13px] text-[#14140F]">{c.label}</span>
						<span class="text-[13px] text-[#5B5C52]">{c.note}</span>
					</div>
					<button
						type="button"
						onclick={() => copy(c.cmd)}
						class="group w-full flex items-center justify-between gap-4 rounded-lg border border-[#E5E3D8] bg-[#F5F4EE] px-4 py-3.5 text-left hover:border-[#14140F] transition-colors"
					>
						<code class="no-scrollbar font-mono text-[12.5px] text-[#14140F] overflow-x-auto whitespace-nowrap">{c.cmd}</code>
						<span class="font-mono text-[11px] text-[#5B5C52] group-hover:text-[#1F7A4C] shrink-0">
							{copied === c.cmd ? 'copied' : 'copy'}
						</span>
					</button>
				</div>
			{/each}
		</div>

		<a
			href="https://github.com/FaizeenHoque/Ziro/wiki"
			target="_blank"
			rel="noreferrer"
			class="inline-block mt-8 font-mono text-[13px] text-[#5B5C52] hover:text-[#14140F] underline underline-offset-4 decoration-[#E5E3D8] transition-colors"
		>
			Full usage guide and keybinds →
		</a>
	</div>
</section>