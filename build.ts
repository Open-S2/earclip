await Bun.build({
  entrypoints: ['./src/index.ts', './src/earcut.ts'],
  outdir: './dist',
  minify: true,
  sourcemap: 'external',
  splitting: true,
  target: 'browser'
})
