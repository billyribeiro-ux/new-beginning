export function toSlug(value: string): string {
	return value
		.toLowerCase()
		.normalize('NFKD')
		.replace(/[̀-ͯ]/g, '')
		.replace(/[^a-z0-9]+/g, '-')
		.replace(/(^-|-$)/g, '');
}

export function relativeTime(iso: string): string {
	const date = new Date(iso);
	const diff = Date.now() - date.getTime();
	const sec = Math.round(diff / 1000);
	if (sec < 60) return `${sec}s ago`;
	const min = Math.round(sec / 60);
	if (min < 60) return `${min}m ago`;
	const hr = Math.round(min / 60);
	if (hr < 24) return `${hr}h ago`;
	const days = Math.round(hr / 24);
	if (days < 30) return `${days}d ago`;
	return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
}

export function formatDate(iso: string, opts?: Intl.DateTimeFormatOptions): string {
	return new Date(iso).toLocaleDateString('en-US', {
		month: 'short',
		day: 'numeric',
		year: 'numeric',
		...opts
	});
}
