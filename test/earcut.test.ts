import { expect, test } from 'bun:test'
import { flatten, deviation } from '../dist/index.js'
import earcut from '../dist/earcut.js'
import fs from 'fs'
import path, { dirname } from 'path'
import { fileURLToPath } from 'url'

const _dirname = dirname(fileURLToPath(import.meta.url))
const expected = JSON.parse(fs.readFileSync(path.join(_dirname, 'expected.json'), 'utf8'))

test('indices-2d', () => {
  const indices = earcut([10, 0, 0, 50, 60, 60, 70, 10])
  expect(indices).toEqual([1, 0, 3, 3, 2, 1])
})

test('indices-3d', () => {
  const indices = earcut([10, 0, 0, 0, 50, 0, 60, 60, 0, 70, 10, 0], undefined, 3)
  expect(indices).toEqual([1, 0, 3, 3, 2, 1])
})

test('empty', () => {
  expect(earcut([])).toEqual([])
})

Object.keys(expected.triangles).forEach((id) => {
  test(id, () => {
    const data = flatten(JSON.parse(fs.readFileSync(path.join(_dirname, '/fixtures/' + id + '.json'), 'utf-8')))
    const indices = earcut(data.vertices, data.holeIndices, data.dim)
    const actualDeviation = deviation(data.vertices, data.holeIndices, data.dim, indices)
    const expectedTriangles = expected.triangles[id]
    // eslint-disable-next-line @typescript-eslint/strict-boolean-expressions
    const expectedDeviation = expected.errors[id] || 0

    const numTriangles = indices.length / 3
    // t.ok(numTriangles === expectedTriangles, numTriangles + ' triangles when expected ' + expectedTriangles)
    expect(numTriangles).toBe(expectedTriangles)

    if (expectedTriangles > 0) {
      // t.ok(actualDeviation <= expectedDeviation,
      //   'deviation ' + actualDeviation + ' <= ' + expectedDeviation)
      expect(actualDeviation).toBeLessThanOrEqual(expectedDeviation)
    }
  })
})

test('infinite-loop', () => {
  const indices = earcut([1, 2, 2, 2, 1, 2, 1, 1, 1, 2, 4, 1, 5, 1, 3, 2, 4, 2, 4, 1], [5], 2)
  expect(indices).toEqual([8, 5, 6])
})
