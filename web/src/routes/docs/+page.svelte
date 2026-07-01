<script lang="ts">
	import { onMount } from 'svelte';
	import Nav from '$lib/components/Nav.svelte';
	import Footer from '$lib/components/Footer.svelte';
	import Callout from '$lib/components/Callout.svelte';
	import CodeBlock from '$lib/components/Codeblock.svelte';
	import DocsTable from '$lib/components/Docstable.svelte';

	const sections = [
		{ id: 'installation', label: 'Installation' },
		{ id: 'opening-files', label: 'Opening Files' },
		{ id: 'modes', label: 'Modes' },
		{ id: 'switching-modes', label: 'Switching Modes' },
		{ id: 'navigation', label: 'Navigation' },
		{ id: 'commands', label: 'Commands' },
		{ id: 'syntax-highlighting', label: 'Syntax Highlighting' }
	];

	let activeId = $state(sections[0].id);

	onMount(() => {
		const observer = new IntersectionObserver(
			(entries) => {
				for (const entry of entries) {
					if (entry.isIntersecting) {
						activeId = entry.target.id;
					}
				}
			},
			{ rootMargin: '-15% 0px -70% 0px', threshold: 0 }
		);

		for (const s of sections) {
			const el = document.getElementById(s.id);
			if (el) observer.observe(el);
		}

		return () => observer.disconnect();
	});
</script>

<svelte:head>
	<title>Docs — Ziro</title>
	<meta name="description" content="Installation, modes, navigation, and commands for Ziro." />
</svelte:head>

<div class="bg-[#FCFCFA] min-h-screen font-sans text-[#14140F]">
	<Nav />

	<div class="mx-auto max-w-5xl px-6 pt-32 pb-24 grid md:grid-cols-[180px_1fr] gap-12">
		<!-- sidebar, styled like the editor status bar -->
		<aside class="hidden md:block">
			<div class="sticky top-28">
				<div class="font-mono text-[11px] text-white bg-[#14140F] rounded-md px-3 py-2 mb-4">
					<div class="text-white/40">ziro docs</div>
					<div class="text-[#7DA98F] mt-0.5">
						{sections.find((s) => s.id === activeId)?.label}
					</div>
				</div>
				<nav class="flex flex-col gap-0.5">
					{#each sections as s (s.id)}
						<a
							href={`#${s.id}`}
							class="font-mono text-[13px] px-3 py-1.5 rounded-md border-l-2 transition-colors"
							class:border-[#1F7A4C]={activeId === s.id}
							class:text-[#14140F]={activeId === s.id}
							class:border-transparent={activeId !== s.id}
							class:text-[#5B5C52]={activeId !== s.id}
						>
							{s.label}
						</a>
					{/each}
				</nav>
			</div>
		</aside>

		<!-- content -->
		<div class="max-w-2xl">
			<p class="font-mono text-[13px] text-[#1F7A4C] mb-3">docs</p>
			<h1 class="font-mono text-[34px] tracking-tight mb-12">Everything you need to run Ziro.</h1>

			<section id="installation" class="scroll-mt-28 mb-16">
				<h2 class="font-mono text-[20px] tracking-tight mb-1">Installation</h2>
				<p class="text-[14px] text-[#5B5C52] mb-4">
					Ziro is currently only supported on <strong class="text-[#14140F]">Linux</strong>.
					Tested on Arch Linux — other distros may work but are untested.
				</p>

				<Callout type="warning">
					macOS and Windows are not supported. There are no plans to change this.
				</Callout>

				<h3 class="font-mono text-[15px] mt-8 mb-1">Stable</h3>
				<p class="text-[14px] text-[#5B5C52]">Downloads a prebuilt binary. No Rust required.</p>
				<CodeBlock
					label="bash"
					code={`curl -sSL https://raw.githubusercontent.com/FaizeenHoque/ziro/main/install.sh | bash`}
				/>

				<h3 class="font-mono text-[15px] mt-8 mb-1">Rolling</h3>
				<p class="text-[14px] text-[#5B5C52]">
					Builds from the latest commit on <code class="font-mono text-[13px] bg-[#F5F4EE] px-1.5 py-0.5 rounded">main</code>. Requires Rust and Cargo.
				</p>
				<CodeBlock
					label="bash"
					code={`curl -sSL https://raw.githubusercontent.com/FaizeenHoque/ziro/main/install-rolling.sh | bash`}
				/>

				<Callout type="note">
					Rolling may be unstable. Use stable unless you want the absolute latest changes.
				</Callout>

				<h3 class="font-mono text-[15px] mt-8 mb-1">Building from source</h3>
				<CodeBlock
					label="bash"
					code={`git clone https://github.com/FaizeenHoque/ziro\ncd ziro\ncargo build --release\nsudo mv target/release/ziro /usr/local/bin/ziro`}
				/>
				<p class="text-[14px] text-[#5B5C52]">
					Requires Rust 1.78+. Install Rust from
					<a href="https://rustup.rs" target="_blank" rel="noreferrer" class="text-[#14140F] underline underline-offset-2">rustup.rs</a>.
				</p>

				<Callout type="note">
					Both install scripts require <code>sudo</code> to move the binary to <code>/usr/local/bin</code>.
					Read the script before running if you're cautious about piping curl to bash.
				</Callout>
			</section>

			<section id="opening-files" class="scroll-mt-28 mb-16">
				<h2 class="font-mono text-[20px] tracking-tight mb-1">Opening Files</h2>
				<CodeBlock
					copyable={false}
					code={`ziro              # open a blank buffer\nziro file.txt     # open a specific file`}
				/>
				<p class="text-[14px] text-[#5B5C52]">
					If you open Ziro without a file, you'll start with a blank buffer. You'll be prompted
					for a filename when you save.
				</p>
			</section>

			<section id="modes" class="scroll-mt-28 mb-16">
				<h2 class="font-mono text-[20px] tracking-tight mb-1">Modes</h2>
				<p class="text-[14px] text-[#5B5C52] mb-4">
					Ziro uses a modal editing system, similar to Vim. There are three modes:
				</p>
				<DocsTable
					headers={['Mode', 'Description']}
					rows={[
						['Normal', 'Navigate the file. Default mode on startup.'],
						['Insert', 'Type and edit text.'],
						['Command', 'Run commands like save and quit.']
					]}
				/>
				<Callout type="tip">
					If you're lost, press <code>Esc</code>. It always takes you back to Normal mode.
				</Callout>
			</section>

			<section id="switching-modes" class="scroll-mt-28 mb-16">
				<h2 class="font-mono text-[20px] tracking-tight mb-1">Switching Modes</h2>
				<DocsTable
					headers={['Key', 'From', 'To']}
					rows={[
						['`i`', 'Normal', 'Insert'],
						['`Esc`', 'Insert', 'Normal'],
						['`:`', 'Normal', 'Command'],
						['`Esc` or `Enter`', 'Command', 'Normal']
					]}
				/>
			</section>

			<section id="navigation" class="scroll-mt-28 mb-16">
				<h2 class="font-mono text-[20px] tracking-tight mb-1">Navigation</h2>
				<p class="text-[14px] text-[#5B5C52] mb-4">Arrow keys work in both Normal and Insert mode.</p>
				<DocsTable
					headers={['Key', 'Action']}
					rows={[
						['`↑`', 'Move up'],
						['`↓`', 'Move down'],
						['`←`', 'Move left'],
						['`→`', 'Move right']
					]}
				/>
			</section>

			<section id="commands" class="scroll-mt-28 mb-16">
				<h2 class="font-mono text-[20px] tracking-tight mb-1">Commands</h2>
				<p class="text-[14px] text-[#5B5C52] mb-4">
					Enter Command mode with <code class="font-mono text-[13px] bg-[#F5F4EE] px-1.5 py-0.5 rounded">:</code>
					from Normal mode, then type a command and press <code class="font-mono text-[13px] bg-[#F5F4EE] px-1.5 py-0.5 rounded">Enter</code>.
				</p>
				<DocsTable
					headers={['Command', 'Action']}
					rows={[
						['`:w`', 'Save the current file'],
						['`:q`', 'Quit (warns if unsaved changes exist)'],
						['`:!q`', 'Force quit without saving'],
						['`:wq`', 'Save and quit'],
						['`:x`', 'Save and quit (same as `:wq`)']
					]}
				/>
				<Callout type="warning">
					<code>:q</code> will refuse to close if you have unsaved changes. Use <code>:!q</code>
					to force quit and discard them, or <code>:wq</code> to save first.
				</Callout>
			</section>

			<section id="syntax-highlighting" class="scroll-mt-28">
				<h2 class="font-mono text-[20px] tracking-tight mb-1">Syntax Highlighting</h2>
				<p class="text-[14px] text-[#5B5C52] mb-4">
					Ziro automatically detects the language from the file extension and applies syntax
					highlighting. Supported languages include anything syntect supports out of the box:
					Rust, Python, JavaScript, TypeScript, Go, C, C++, HTML, CSS, JSON, TOML, Markdown, and
					more.
				</p>
				<Callout type="note">
					If you open Ziro without a filename, syntax highlighting won't activate until you save
					the file with a name.
				</Callout>
			</section>
		</div>
	</div>

	<Footer />
</div>