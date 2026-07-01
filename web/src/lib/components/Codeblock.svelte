<script lang="ts">
	let { code, label = '', copyable = true } = $props<{
		code: string;
		label?: string;
		copyable?: boolean;
	}>();

	let copied = $state(false);

	async function copy() {
		try {
			await navigator.clipboard.writeText(code);
			copied = true;
			setTimeout(() => (copied = false), 1600);
		} catch {
			// clipboard unavailable, fail silently
		}
	}

	const lines = code.split('\n');
</script>

<div class="my-5 rounded-lg border border-[#E5E3D8] bg-[#14140F] overflow-hidden">
	{#if label || copyable}
		<div class="flex items-center justify-between px-4 h-8 border-b border-white/10">
			<span class="font-mono text-[11px] text-white/40">{label}</span>
			{#if copyable}
				<button
					type="button"
					onclick={copy}
					class="font-mono text-[11px] text-white/40 hover:text-[#1F7A4C] transition-colors"
				>
					{copied ? 'copied' : 'copy'}
				</button>
			{/if}
		</div>
	{/if}
	<pre class="px-4 py-3.5 overflow-x-auto"><code class="font-mono text-[12.5px] leading-relaxed text-white/80">{#each lines as line, i (i)}{line}{i < lines.length - 1 ? '\n' : ''}{/each}</code></pre>
</div>