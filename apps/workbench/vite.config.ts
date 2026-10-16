import tailwindcss from '@tailwindcss/vite';
import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';

/** The plugins the workbench draws the app with: Tailwind for the `ui-` utilities, and React. */
export default defineConfig({
	plugins: [tailwindcss(), react()],
});
