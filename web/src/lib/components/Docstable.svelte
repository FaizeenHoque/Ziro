<script lang="ts">
	let { headers, rows } = $props<{
		headers: string[];
		rows: string[][];
	}>();

	function parseInline(cell: string) {
		const escaped = cell
			.replace(/&/g, '&amp;')
			.replace(/</g, '&lt;')
			.replace(/>/g, '&gt;');
		return escaped.replace(
			/`([^`]+)`/g,
			'<code class="font-mono text-[12.5px] bg-[#F5F4EE] px-1.5 py-0.5 rounded text-[#14140F]">$1</code>'
		);
	}
</script>

<div class="my-5 overflow-x-auto">
	<table class="w-full border-collapse">
		<thead>
			<tr class="border-b border-[#E5E3D8]">
				{#each headers as h (h)}
					<th class="text-left font-mono text-[11px] uppercase tracking-wide text-[#5B5C52] pb-2 pr-6 whitespace-nowrap">
						{h}
					</th>
				{/each}
			</tr>
		</thead>
		<tbody>
			{#each rows as row, i (i)}
				<tr class="border-b border-[#E5E3D8]/60">
					{#each row as cell, j (j)}
						<td class="text-[14px] text-[#14140F] py-2.5 pr-6 whitespace-nowrap">
							{@html parseInline(cell)}
						</td>
					{/each}
				</tr>
			{/each}
		</tbody>
	</table>
</div>