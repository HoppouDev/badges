import { error } from '@sveltejs/kit';
import { renderDoc, slugs } from '#lib/server/docs.ts';
import type { EntryGenerator, PageServerLoad } from './$types';

// Docs are rendered to static HTML at build time
export const prerender = true;

export const entries: EntryGenerator = () => slugs.map((slug) => ({ slug }));

export const load: PageServerLoad = async ({ params }) => {
	const doc = await renderDoc(params.slug);
	if (!doc) error(404, 'Not found');
	return doc;
};
