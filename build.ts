import bun from 'bun';

try {
  console.info('Starting the build process...');
  await bun.build({
    entrypoints: ['src/index.ts'],
    outdir: 'dist',
    format: 'esm',
    minify: true,
    sourcemap: 'external',
    // target: 'esnext', // Adjust target based on your project needs
  });
  console.info('Build completed successfully!');
} catch (error) {
  console.error('Build failed:', error);
}
