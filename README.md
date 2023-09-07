# earclip [![npm][npm-image]][npm-url] [![downloads][downloads-image]][downloads-url] [![bundlephobia][bundlephobia-image]][bundlephobia-url] [![JavaScript Style Guide](https://img.shields.io/badge/code_style-standard-brightgreen.svg)](https://standardjs.com)

[npm-image]: https://img.shields.io/npm/v/earclip.svg
[npm-url]: https://npmjs.org/package/earclip
[bundlephobia-image]: https://img.shields.io/bundlephobia/minzip/earclip.svg
[bundlephobia-url]: https://bundlephobia.com/package/earclip
[downloads-image]: https://img.shields.io/npm/dm/earclip.svg
[downloads-url]: https://www.npmjs.com/package/earclip

## About

The fastest and smallest JavaScript polygon triangulation library with builtin tesselation. 2.9
kB minified and gzipped.

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

## Why another triangulation library?

The aim of this project is to create a JS triangulation library
that is **fast enough for real-time triangulation in the browser**,
sacrificing triangulation quality for raw speed and simplicity,
while being robust enough to handle most practical datasets without crashing or producing garbage.
Some benchmarks using Node 0.12:

(ops/sec)         | pts  | earcut    | libtess  | poly2tri | pnltri    | polyk
------------------| ---- | --------- | -------- | -------- | --------- | ------
OSM building      | 15   | _795,935_ | _50,640_ | _61,501_ | _122,966_ | _175,570_
dude shape        | 94   | _35,658_  | _10,339_ | _8,784_  | _11,172_  | _13,557_
holed dude shape  | 104  | _28,319_  | _8,883_  | _7,494_  | _2,130_   | n/a
complex OSM water | 2523 | _543_     | _77.54_  | failure  | failure   | n/a
huge OSM water    | 5667 | _95_      | _29.30_  | failure  | failure   | n/a

If you want to get correct triangulation even on very bad data with lots of self-intersections
and earcut is not precise enough, take a look at [libtess.js](https://github.com/brendankenny/libtess.js).

## Usage

```js
import { earclip } from 'earclip'

const poly = [[[3506,-2048],[7464,402],[-2048,2685],[-2048,-2048],[3506,-2048]],[[-2048,-37],[1235,747],[338,-1464],[-116,-1188],[-2048,-381],[-2048,-37]],[[-1491,-1981],[-1300,-1800],[-1155,-1981],[-1491,-1981]]]
const modulo = 8192 / 2

const res = earclip(poly, modulo)
console.log(res)
```
