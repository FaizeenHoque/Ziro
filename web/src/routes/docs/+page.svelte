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
		{ id: 'keybindings', label: 'Keybindings' },
		{ id: 'undo-redo', label: 'Undo & Redo' },
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
	<meta name="description" content="Installation, keybindings, and behavior for Ziro." />
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

			<Callout type="warning">
				Ziro is early. There is no modal editing yet — no Normal/Insert/Command
				split. What's documented below is what's actually wired up in the
				codebase today, not the roadmap.
			</Callout>

			<section id="installation" class="scroll-mt-28 mb-16 mt-12">
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
					for a filename the first time you save.
				</p>
			</section>

			<section id="modes" class="scroll-mt-28 mb-16">
				<h2 class="font-mono text-[20px] tracking-tight mb-1">Modes</h2>
				<p class="text-[14px] text-[#5B5C52] mb-4">
					Ziro does not currently have a Vim-style Normal/Insert/Command split.
					Typed characters insert directly wherever your cursor is — there's no
					mode switch required, and none available yet.
				</p>
				<p class="text-[14px] text-[#5B5C52] mb-4">
					The one exception is the <strong class="text-[#14140F]">filename prompt</strong>,
					a popup that appears when you save a buffer that has no file path yet
					(<code class="font-mono text-[13px] bg-[#F5F4EE] px-1.5 py-0.5 rounded">Ctrl+S</code>).
					While it's open, typing edits the filename instead of the document.
				</p>
				<Callout type="warning">
					A full modal system (Normal / Insert / Command, plus <code>:w</code>,
					<code>:q</code>, <code>:wq</code>-style commands) is planned but not
					implemented. If you see references to it elsewhere, they're aspirational.
				</Callout>
			</section>

			<section id="keybindings" class="scroll-mt-28 mb-16">
				<h2 class="font-mono text-[20px] tracking-tight mb-1">Keybindings</h2>
				<p class="text-[14px] text-[#5B5C52] mb-4">
					These work anywhere in the document (not mode-gated), except where noted.
				</p>
				<DocsTable
					headers={['Key', 'Action']}
					rows={[
						['`Ctrl+S`', 'Save. Opens the filename popup if the buffer has no path yet.'],
						['`Ctrl+W`', 'Quit. Refuses and shows a status message if there are unsaved changes.'],
						['`Ctrl+Alt+W`', 'Force quit, discarding any unsaved changes.'],
						['`Ctrl+U`', 'Undo the last change.'],
						['`Ctrl+R`', 'Redo the last undone change.'],
						['`↑ ↓ ← →`', 'Move the cursor.'],
						['`Enter`', 'Insert a newline, splitting the current line.'],
						['`Backspace`', 'Delete the character before the cursor.'],
						['`Esc`', 'Cancel the filename popup (only active while it\'s open).']
					]}
				/>
				<Callout type="tip">
					<code>Ctrl+W</code> won't let you lose work silently — if the buffer is
					dirty it just shows a status message instead of quitting. Use
					<code>Ctrl+Alt+W</code> if you actually want to discard changes.
				</Callout>
			</section>

			<section id="undo-redo" class="scroll-mt-28 mb-16">
				<h2 class="font-mono text-[20px] tracking-tight mb-1">Undo & Redo</h2>
				<p class="text-[14px] text-[#5B5C52] mb-4">
					Ziro snapshots the full buffer and cursor position before each edit,
					not per-keystroke. Consecutive character insertions and consecutive
					backspaces are grouped into a single undo step; typing a word and
					then deleting it are still two separate steps.
				</p>
				<DocsTable
					headers={['Key', 'Action']}
					rows={[
						['`Ctrl+U`', 'Undo'],
						['`Ctrl+R`', 'Redo']
					]}
				/>
				<Callout type="note">
					Redo history is cleared as soon as you make a new edit after undoing —
					same behavior you'd expect from any standard editor.
				</Callout>
			</section>

			<section id="syntax-highlighting" class="scroll-mt-28">
				<h2 class="font-mono text-[20px] tracking-tight mb-1">Syntax Highlighting</h2>
				<p class="text-[14px] text-[#5B5C52] mb-4">
					Ziro detects the language from the file's extension and applies syntax
					highlighting via <code class="font-mono text-[13px] bg-[#F5F4EE] px-1.5 py-0.5 rounded">syntect</code>,
					using the Solarized (dark) theme. Supported languages include anything
					syntect ships by default: Rust, Python, JavaScript, TypeScript, Go, C,
					C++, HTML, CSS, JSON, TOML, Markdown, and more.
				</p>
				<Callout type="note">
					If you open Ziro without a filename, there's no extension to detect —
					highlighting falls back to plain text until you save the buffer with
					a real name.
				</Callout>
			</section>
		</div>
	</div>

	<Footer />
</div>