import rehypeSlug from 'rehype-slug';
import rehypeStringify from 'rehype-stringify';
import remarkGfm from 'remark-gfm';
import remarkParse from 'remark-parse';
import remarkRehype from 'remark-rehype';
import { unified } from 'unified';

const DOCS_DIR = '/src/docs/';

/** Markdown sources, bundled at build time */
const sources = import.meta.glob<string>('/src/docs/**/*.md', {
	query: '?raw',
	import: 'default',
	eager: true
});

/** Markdown (with GitHub tables) to HTML; raw HTML in the markdown is dropped */
const processor = unified()
	.use(remarkParse)
	.use(remarkGfm)
	.use(remarkRehype)
	.use(rehypeSlug)
	.use(rehypeStringify);

/** URL path below `/docs` for a doc file: `index.md` is `/docs`, `a/b.md` is `/docs/a/b` */
function slugOf(path: string): string {
	const name = path.slice(DOCS_DIR.length, -'.md'.length);
	return name === 'index' ? '' : name.replace(/\/index$/, '');
}

const paths = new Map(Object.keys(sources).map((path) => [slugOf(path), path]));

export const slugs = [...paths.keys()];

export interface Doc {
	title: string;
	html: string;
}

/** Render the doc at `slug`, or `undefined` if there is none */
export async function renderDoc(slug: string): Promise<Doc | undefined> {
	const path = paths.get(slug);
	if (path === undefined) return undefined;
	const markdown = sources[path];
	const title = /^#\s+(.+?)\s*$/m.exec(markdown)?.[1] ?? slug;
	return { title, html: String(await processor.process(markdown)) };
}
