import earcut from '../src/earcut';

import { deviation, flatten } from '../src/index';
import { expect, test } from 'bun:test';

const expected = await Bun.file(`${__dirname}/expected.json`).json();

test('indices-2d', () => {
  const indices = earcut([10, 0, 0, 50, 60, 60, 70, 10]);
  expect(indices).toEqual([1, 0, 3, 3, 2, 1]);
});

test('indices-3d', () => {
  const indices = earcut([10, 0, 0, 0, 50, 0, 60, 60, 0, 70, 10, 0], undefined, 3);
  expect(indices).toEqual([1, 0, 3, 3, 2, 1]);
});

test('empty', async () => {
  expect(earcut([])).toEqual([]);
});

Object.keys(expected.triangles).forEach((id) => {
  test(id, async () => {
    const data = flatten(await Bun.file(`${__dirname}/fixtures/${id}.json`).json());
    const indices = earcut(data.vertices, data.holeIndices, data.dim);
    const actualDeviation = deviation(data.vertices, data.holeIndices, data.dim, indices);
    const expectedTriangles = expected.triangles[id];

    const expectedDeviation = expected.errors[id] || 0;

    const numTriangles = indices.length / 3;
    expect(numTriangles).toBe(expectedTriangles);

    if (expectedTriangles > 0) {
      expect(actualDeviation).toBeLessThanOrEqual(expectedDeviation);
    }
  });
});

test('infinite-loop', () => {
  const indices = earcut([1, 2, 2, 2, 1, 2, 1, 1, 1, 2, 4, 1, 5, 1, 3, 2, 4, 2, 4, 1], [5], 2);
  expect(indices).toEqual([8, 5, 6]);
});
