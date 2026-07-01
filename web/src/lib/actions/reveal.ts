export function reveal(node: HTMLElement) {
	node.style.opacity = '0';
	node.style.transform = 'translateY(14px)';
	node.style.transition = 'opacity 0.6s ease, transform 0.6s ease';

	const observer = new IntersectionObserver(
		(entries) => {
			for (const entry of entries) {
				if (entry.isIntersecting) {
					node.style.opacity = '1';
					node.style.transform = 'translateY(0)';
					observer.unobserve(node);
				}
			}
		},
		{ threshold: 0.15 }
	);

	observer.observe(node);

	return {
		destroy() {
			observer.disconnect();
		}
	};
}