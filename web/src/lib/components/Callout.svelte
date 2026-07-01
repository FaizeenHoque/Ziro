<script lang="ts">
	let { type = 'note', children } = $props<{
		type?: 'warning' | 'note' | 'tip';
		children: () => unknown;
	}>();

	const config = {
		warning: { label: 'warning', icon: '▲' },
		note: { label: 'note', icon: '»' },
		tip: { label: 'tip', icon: '→' }
	};
</script>

<div
	class="rounded-lg px-4 py-3.5 my-5 border-l-2"
	class:border-[#14140F]={type === 'warning'}
	class:bg-[#F5F4EE]={type === 'warning'}
	class:border-[#1F7A4C]={type !== 'warning'}
	class:bg-[#F0F5F1]={type === 'note'}
	class:bg-[#EAF4EE]={type === 'tip'}
>
	<div class="flex items-center gap-2 mb-1">
		<span
			class="font-mono text-[11px]"
			class:text-[#14140F]={type === 'warning'}
			class:text-[#1F7A4C]={type !== 'warning'}
		>
			{config[type as keyof typeof config].icon}
		</span>
		<span
			class="font-mono text-[11px] uppercase tracking-wide"
			class:text-[#14140F]={type === 'warning'}
			class:text-[#1F7A4C]={type !== 'warning'}
		>
			{config[type as keyof typeof config].label}
		</span>
	</div>
	<div class="text-[14px] leading-relaxed text-[#14140F] [&_code]:font-mono [&_code]:text-[13px] [&_code]:bg-white/60 [&_code]:px-1 [&_code]:py-0.5 [&_code]:rounded [&_a]:underline [&_a]:underline-offset-2">
		{@render children()}
	</div>
</div>