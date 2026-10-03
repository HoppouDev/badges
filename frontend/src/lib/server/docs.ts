import rehypePrettyCode from 'rehype-pretty-code';
import rehypeSlug from 'rehype-slug';
import rehypeStringify from 'rehype-stringify';
import remarkGfm from 'remark-gfm';
import remarkParse from 'remark-parse';
import remarkRehype from 'remark-rehype';
import { unified } from 'unified';
import { VFile } from 'vfile';
import { matter } from 'vfile-matter';

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
	// Shiki colours for fenced code; both themes are inlined as CSS variables and
	// layout.css picks one from the .dark class, so there is no flash on load
	.use(rehypePrettyCode, { theme: { light: 'catppuccin-latte', dark: 'catppuccin-mocha' } })
	.use(rehypeSlug)
	.use(rehypeStringify);

/** Frontmatter every doc must start with */
interface Frontmatter {
	/** Sidebar link and page title */
	title: string;
	/** Sidebar group the page is listed under */
	section: string;
	/** Position among pages and sections, lowest first (default 0; ties sort by title) */
	order: number;
}

interface Source {
	slug: string;
	meta: Frontmatter;
	/** Markdown with the frontmatter removed */
	file: VFile;
}

/** URL path below `/docs` for a doc file: `index.md` is `/docs`, `a/b.md` is `/docs/a/b` */
function slugOf(path: string): string {
	const name = path.slice(DOCS_DIR.length, -'.md'.length);
	return name === 'index' ? '' : name.replace(/\/index$/, '');
}

/** Read and validate a doc's frontmatter, failing the build on mistakes */
function load(path: string, markdown: string): Source {
	const file = new VFile({ path, value: markdown });
	matter(file, { strip: true });
	const data = file.data.matter as Record<string, unknown>;
	const text = (key: string) => {
		const value = data[key];
		if (typeof value !== 'string' || !value.trim()) {
			throw new Error(`${path}: frontmatter needs a "${key}"`);
		}
		return value.trim();
	};
	const order = data.order ?? 0;
	// YAML .nan/.inf are numbers too, and NaN would scramble the sort
	if (typeof order !== 'number' || !Number.isFinite(order)) {
		throw new Error(`${path}: frontmatter "order" must be a finite number`);
	}
	return {
		slug: slugOf(path),
		meta: { title: text('title'), section: text('section'), order },
		file
	};
}

const docs = Object.entries(sources)
	.map(([path, markdown]) => load(path, markdown))
	.sort((a, b) => a.meta.order - b.meta.order || a.meta.title.localeCompare(b.meta.title));

// a.md and a/index.md both map to /docs/a; only one could be served
const bySlug = new Map<string, Source>();
for (const doc of docs) {
	const other = bySlug.get(doc.slug);
	if (other)
		throw new Error(`${other.file.path} and ${doc.file.path} both map to /docs/${doc.slug}`);
	bySlug.set(doc.slug, doc);
}

export const slugs = docs.map((doc) => doc.slug);

export interface NavSection {
	title: string;
	pages: { title: string; href: string }[];
}

/** Sidebar sections in the order of their first page */
export const nav: NavSection[] = [];
for (const { slug, meta } of docs) {
	let section = nav.find((s) => s.title === meta.section);
	if (!section) {
		section = { title: meta.section, pages: [] };
		nav.push(section);
	}
	section.pages.push({ title: meta.title, href: slug ? `/docs/${slug}` : '/docs' });
}

export interface Doc {
	title: string;
	html: string;
}

/** Render the doc at `slug`, or `undefined` if there is none */
export async function renderDoc(slug: string): Promise<Doc | undefined> {
	const doc = bySlug.get(slug);
	if (doc === undefined) return undefined;
	const file = new VFile({ path: doc.file.path, value: doc.file.value });
	const tree = processor.parse(file);
	// The page heading comes from the frontmatter title; a second one would duplicate it
	if (tree.children.some((node) => node.type === 'heading' && node.depth === 1)) {
		throw new Error(`${file.path}: use the frontmatter "title" instead of a "# " heading`);
	}
	const html = processor.stringify(await processor.run(tree, file), file);
	return { title: doc.meta.title, html };
}
