<h1 style="text-align: center;">
    <div align="center">earclip</div>
</h1>

<p align="center">
  <img src="https://img.shields.io/github/actions/workflow/status/Open-S2/earclip/test.yml?logo=github" alt="GitHub Actions Workflow Status">
  <a href="https://npmjs.org/package/earclip">
    <img src="https://img.shields.io/npm/v/earclip.svg?logo=npm&logoColor=white" alt="npm">
  </a>
  <a href="https://crates.io/crates/earclip">
    <img src="https://img.shields.io/crates/v/earclip.svg?logo=rust&logoColor=white" alt="crate">
  </a>
  <a href="https://www.npmjs.com/package/earclip">
    <img src="https://img.shields.io/npm/dm/earclip.svg" alt="downloads">
  </a>
  <a href="https://bundlejs.com/?q=earclip&treeshake=%5B%7B+earclip+%7D%5D">
    <img src="https://img.shields.io/bundlejs/size/earclip?exports=earclip" alt="bundle">
  </a>
  <a href="https://open-s2.github.io/earclip/">
    <img src="https://img.shields.io/badge/docs-typescript-yellow.svg" alt="docs-ts">
  </a>
  <a href="https://docs.rs/earclip">
    <img src="https://img.shields.io/badge/docs-rust-yellow.svg" alt="docs-rust">
  </a>
  <img src="https://raw.githubusercontent.com/Open-S2/earclip/master/assets/doc-coverage.svg" alt="doc-coverage">
  <a href="https://coveralls.io/github/Open-S2/earclip?branch=master">
    <img src="https://coveralls.io/repos/github/Open-S2/earclip/badge.svg?branch=master" alt="code-coverage">
  </a>
  <a href="https://discord.opens2.com">
    <img src="https://img.shields.io/discord/953563031701426206?logo=discord&logoColor=white" alt="Discord">
  </a>
</p>

## About

The fastest and smallest JavaScript polygon triangulation library with builtin tesselation. 3.18 kB minified and gzipped.

## Install

```bash
# NPM
npm install earcut
# PNPM
pnpm add earcut
# Yarn
yarn add earcut
# Bun
bun add earcut
```

## The Algorithm

The library implements a modified ear slicing algorithm,
optimized by [z-order curve](http://en.wikipedia.org/wiki/Z-order_curve) hashing
and extended to handle holes, twisted polygons, degeneracies and self-intersections
in a way that doesn't _guarantee_ correctness of triangulation,
but attempts to always produce acceptable results for practical data.

It's based on ideas from
[FIST: Fast Industrial-Strength Triangulation of Polygons](http://www.cosy.sbg.ac.at/~held/projects/triang/triang.html) by Martin Held
and [Triangulation by Ear Clipping](http://www.geometrictools.com/Documentation/TriangulationByEarClipping.pdf) by David Eberly.

## Usage

```ts
import { earclip } from 'earclip'

const poly = [[[3506,-2048],[7464,402],[-2048,2685],[-2048,-2048],[3506,-2048]],[[-2048,-37],[1235,747],[338,-1464],[-116,-1188],[-2048,-381],[-2048,-37]],[[-1491,-1981],[-1300,-1800],[-1155,-1981],[-1491,-1981]]]
const modulo = 8192 / 2

const res = earclip(poly, modulo)
console.log(res)

const polyAsPoints = [
    [{ x: 3506, y: -2048 },{ x: 7464, y: 402 },{ x: -2048, y: 2685 },{ x: -2048, y: -2048 },{ 3506, y: -2048 }],
    [{ x: -2048, y: -37 },{ x: 1235, y: 747 },{ x: 338, y: -1464 },{ x: -116, y: -1188 },{ x: -2048, y: -381 },{ x: -2048, y: -37 }],
    [{ x: -1491, y: -1981 },{ x: -1300, y: -1800 },{ x: -1155, y: -1981 },{ x: -1491, y: -1981 }],
]
const res2 = earclip(polyAsPoints, modulo)

assert(res === res2)
```

> [!NOTE]  
> Safety Unsafe code is forbidden by a #![forbid(unsafe_code)] attribute in the root of the rust library.

---

## Development

### Requirements

You need the tool `tarpaulin` to generate the coverage report. Install it using the following command:

```bash
cargo install cargo-tarpaulin
```

The `bacon coverage` tool is used to generate the coverage report. To utilize the [pycobertura](https://pypi.org/project/pycobertura/) package for a prettier coverage report, install it using the following command:

```bash
pip install pycobertura
```

### Running Tests

To run the tests, use the following command:

```bash
# TYPESCRIPT
## basic test
bun run test
## live testing
bun run test:dev

# RUST
## basic test
cargo test
# live testing
bacon test
```

### Generating Coverage Report

To generate the coverage report, use the following command:

```bash
cargo tarpaulin
# faster
cargo tarpaulin --color always --skip-clean
# bacon
bacon coverage # or type `l` inside the tool
```

## Benchmarks

### Rust

Run the Rust benchmarks using the following command:

```bash
cargo +nightly bench
```

<!-- ### Generating Coverage Report

```bash -->
