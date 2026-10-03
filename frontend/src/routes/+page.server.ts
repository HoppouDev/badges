import { redirect } from '@sveltejs/kit';

// The docs are the site until there is a homepage
export const load = () => redirect(307, '/docs');
