// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import react from '@astrojs/react';

// https://astro.build/config
export default defineConfig({
	integrations: [
		starlight({
			title: 'Rescope',
			description: 'A local Descope emulator for offline development and testing',
			logo: {
				light: './src/assets/logo.svg',
				dark: './src/assets/logo.svg',
				replacesTitle: true,
			},
			social: [
				{ icon: 'github', label: 'GitHub', href: 'https://github.com/descope/rescope' },
			],
			customCss: ['./src/styles/rescope.css'],
			sidebar: [
				{
					label: '🚀 Live Demo',
					items: [
						{ label: 'Interactive Sandbox', slug: 'demo' },
					],
				},
				{
					label: 'Getting Started',
					items: [
						{ label: 'Introduction', slug: 'getting-started/introduction' },
						{ label: 'Installation', slug: 'getting-started/installation' },
						{ label: 'Configuration', slug: 'getting-started/configuration' },
					],
				},
				{
					label: 'Guides',
					items: [
						{ label: 'OTP Authentication', slug: 'guides/otp-authentication' },
						{ label: 'Magic Links', slug: 'guides/magic-links' },
						{ label: 'Password Auth', slug: 'guides/password-auth' },
						{ label: 'User Management', slug: 'guides/user-management' },
					],
				},
				{
					label: 'API Reference',
					items: [
						{ label: 'Overview', slug: 'api-reference/overview' },
					{ label: 'Feature Parity', slug: 'feature-parity' },
					],
				},
				{
					label: 'Playground',
					items: [
						{ label: 'Interactive Playground', slug: 'playground' },
					],
				},
			],
		}),
		react(),
	],
});
