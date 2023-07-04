import test from 'tape'
import { flatten, deviation } from '../dist/index.js'
import earcut from '../dist/earcut.js'
import fs from 'fs'
import path, { dirname } from 'path'
import { fileURLToPath } from 'url'

const __dirname = dirname(fileURLToPath(import.meta.url))
const expected = JSON.parse(fs.readFileSync(path.join(__dirname, 'expected.json'), 'utf8'))

test('indices-2d', function (t) {
  const indices = earcut([10, 0, 0, 50, 60, 60, 70, 10])
  t.same(indices, [1, 0, 3, 3, 2, 1])
  t.end()
})

test('indices-3d', function (t) {
  const indices = earcut([10, 0, 0, 0, 50, 0, 60, 60, 0, 70, 10, 0], undefined, 3)
  t.same(indices, [1, 0, 3, 3, 2, 1])
  t.end()
})

test('empty', function (t) {
  t.same(earcut([]), [])
  t.end()
})

Object.keys(expected.triangles).forEach(function (id) {
  test(id, function (t) {
    const data = flatten(JSON.parse(fs.readFileSync(path.join(__dirname, '/fixtures/' + id + '.json'))))
    const indices = earcut(data.vertices, data.holeIndices, data.dim)
    const actualDeviation = deviation(data.vertices, data.holeIndices, data.dim, indices)
    const expectedTriangles = expected.triangles[id]
    const expectedDeviation = expected.errors[id] || 0

    const numTriangles = indices.length / 3
    t.ok(numTriangles === expectedTriangles, numTriangles + ' triangles when expected ' + expectedTriangles)

    if (expectedTriangles > 0) {
      t.ok(actualDeviation <= expectedDeviation,
        'deviation ' + actualDeviation + ' <= ' + expectedDeviation)
    }

    t.end()
  })
})

test('infinite-loop', function (t) {
  earcut([1, 2, 2, 2, 1, 2, 1, 1, 1, 2, 4, 1, 5, 1, 3, 2, 4, 2, 4, 1], [5], 2)
  t.end()
})
