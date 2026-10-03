import { nav } from '#lib/server/docs.ts';
import type { LayoutServerLoad } from './$types';

export const prerender = true;

export const load: LayoutServerLoad = () => ({ nav });
