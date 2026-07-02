<script lang="ts">
	import ziroMark from '$lib/assets/ziro_mark.svg';
</script>

<footer class="px-6 py-10 border-t border-[#E5E3D8]">
	<div class="mx-auto max-w-5xl flex flex-col sm:flex-row items-start sm:items-center justify-between gap-6">
		<div class="flex items-center gap-2.5">
			<img src={ziroMark} alt="Ziro" class="h-5 w-5 rounded-[4px]" />
			<span class="font-mono text-[13px] text-[#5B5C52]">MIT © 2026 Faizeen Hoque</span>
		</div>

		<div class="flex items-center gap-6">
			<a
				href="https://github.com/FaizeenHoque/ziro"
				target="_blank"
				rel="noreferrer"
				class="font-mono text-[13px] text-[#5B5C52] hover:text-[#14140F] transition-colors"
			>
				GitHub
			</a>
			<a
				href="https://ziro.faizeenhoque.dev/docs/"
				target="_blank"
				rel="noreferrer"
				class="font-mono text-[13px] text-[#5B5C52] hover:text-[#14140F] transition-colors"
			>
				Wiki
			</a>
			<a
				href="https://github.com/FaizeenHoque/Ziro/releases"
				target="_blank"
				rel="noreferrer"
				class="font-mono text-[13px] text-[#5B5C52] hover:text-[#14140F] transition-colors"
			>
				Releases
			</a>
		</div>
	</div>
</footer>