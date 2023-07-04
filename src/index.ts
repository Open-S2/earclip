import earcut from './earcut.js'

export interface EarclipResult {
  vertices: number[]
  indices: number[]
}

export interface FlattenResult {
  vertices: number[]
  holeIndices: number[]
  dim: number
}

export function earclip (
  polygon: number[][][],
  modulo = Infinity,
  offset = 0
): EarclipResult {
  // Use earcut to build standard triangle set
  const { vertices, holeIndices, dim } = flatten(polygon) // dim => dimensions
  const indices = earcut(vertices, holeIndices, dim)
  // tesselate if necessary
  if (modulo !== Infinity) tesselate(vertices, indices, modulo, dim)
  // update offset and return
  return { vertices, indices: indices.map(index => index + offset) }
}

export function tesselate (
  vertices: number[],
  indices: number[],
  modulo: number,
  dim: number
): void {
  // for each triangle, ensure each triangle line does not pass through iterations of the modulo for x, y, and z
  let A, B, C
  for (let axis = 0; axis < dim; axis++) {
    for (let i = 0; i < indices.length; i += 3) {
      // get indexes of each vertex
      A = indices[i]
      B = indices[i + 1]
      C = indices[i + 2]
      const triangle = splitIfNecessary(A, B, C, vertices, indices, dim, axis, modulo)
      if (triangle !== undefined) {
        indices[i] = triangle[0]
        indices[i + 1] = triangle[1]
        indices[i + 2] = triangle[2]
        i -= 3
      }
    }
  }
}

// given vertices, and an axis of said vertices:
// find a number "x" that is x % modulo === 0 and between v1 and v2
function splitIfNecessary (
  i1: number,
  i2: number,
  i3: number,
  vertices: number[],
  indices: number[],
  dim: number,
  axis: number,
  modulo: number
): [number, number, number] | undefined {
  const v1 = vertices[i1 * dim + axis]
  const v2 = vertices[i2 * dim + axis]
  const v3 = vertices[i3 * dim + axis]
  // 1 is corner
  if (v1 < v2 && v1 < v3) {
    const modPoint = v1 + modulo - mod2(v1, modulo)
    if (modPoint > v1 && modPoint <= v2 && modPoint <= v3 && (v2 !== modPoint || v2 !== modPoint)) {
      return splitRight(modPoint, i1, i2, i3, v1, v2, v3, vertices, indices, dim, axis, modulo)
    }
  } else if (v1 > v2 && v1 > v3) {
    let mod = mod2(v1, modulo)
    if (mod === 0) mod = modulo
    const modPoint = v1 - mod
    if (modPoint < v1 && modPoint >= v2 && modPoint >= v3 && (v2 !== modPoint || v2 !== modPoint)) {
      return splitLeft(modPoint, i1, i2, i3, v1, v2, v3, vertices, indices, dim, axis, modulo)
    }
  }
  // 2 is corner
  if (v2 < v1 && v2 < v3) {
    const modPoint = v2 + modulo - mod2(v2, modulo)
    if (modPoint > v2 && modPoint <= v3 && modPoint <= v1 && (v1 !== modPoint || v3 !== modPoint)) {
      return splitRight(modPoint, i2, i3, i1, v2, v3, v1, vertices, indices, dim, axis, modulo)
    }
  } else if (v2 > v1 && v2 > v3) {
    let mod = mod2(v2, modulo)
    if (mod === 0) mod = modulo
    const modPoint = v2 - mod
    if (modPoint < v2 && modPoint >= v3 && modPoint >= v1 && (v1 !== modPoint || v3 !== modPoint)) {
      return splitLeft(modPoint, i2, i3, i1, v2, v3, v1, vertices, indices, dim, axis, modulo)
    }
  }
  // 3 is corner
  if (v3 < v1 && v3 < v2) {
    const modPoint = v3 + modulo - mod2(v3, modulo)
    if (modPoint > v3 && modPoint <= v1 && modPoint <= v2 && (v1 !== modPoint || v2 !== modPoint)) {
      return splitRight(modPoint, i3, i1, i2, v3, v1, v2, vertices, indices, dim, axis, modulo)
    }
  } else if (v3 > v1 && v3 > v2) {
    let mod = mod2(v3, modulo)
    if (mod === 0) mod = modulo
    const modPoint = v3 - mod
    if (modPoint < v3 && modPoint >= v1 && modPoint >= v2 && (v1 !== modPoint || v2 !== modPoint)) {
      return splitLeft(modPoint, i3, i1, i2, v3, v1, v2, vertices, indices, dim, axis, modulo)
    }
  }
}

function createVertex (
  splitPoint: number,
  i1: number,
  i2: number,
  v1: number,
  v2: number,
  vertices: number[],
  dim: number,
  axis: number
): number {
  const index = vertices.length / dim
  const travelDivisor = (v2 - v1) / (splitPoint - v1)
  let va1, va2
  for (let i = 0; i < dim; i++) {
    va1 = vertices[i1 * dim + i]
    va2 = vertices[i2 * dim + i]
    if (i !== axis) vertices.push(va1 + ((va2 - va1) / travelDivisor))
    else vertices.push(splitPoint)
  }
  return index
}

// i1 is always the vertex with an acute angle.
// splitRight means we start on the left side of this "1D" observation moving right
function splitRight (
  modPoint: number,
  i1: number,
  i2: number,
  i3: number,
  v1: number,
  v2: number,
  v3: number,
  vertices: number[],
  indices: number[],
  dim: number,
  axis: number,
  modulo: number
): [number, number, number] {
  // first case is a standalone triangle
  let i12 = createVertex(modPoint, i1, i2, v1, v2, vertices, dim, axis)
  let i13 = createVertex(modPoint, i1, i3, v1, v3, vertices, dim, axis)
  indices.push(i1, i12, i13)
  modPoint += modulo
  if (v2 < v3) {
    // create lines up to i2
    while (modPoint < v2) {
      // next triangles are i13->i12->nexti13 and nexti13->i12->nexti12 so store in necessary order
      indices.push(i13, i12)
      i13 = createVertex(modPoint, i1, i3, v1, v3, vertices, dim, axis)
      indices.push(i13, i13, i12)
      i12 = createVertex(modPoint, i1, i2, v1, v2, vertices, dim, axis)
      indices.push(i12)
      // increment
      modPoint += modulo
    }
    // add v2 triangle if necessary
    indices.push(i13, i12, i2)
    // return the remaining triangle
    return [i13, i2, i3]
  } else {
    // create lines up to i2
    while (modPoint < v3) {
      // next triangles are i13->i12->nexti13 and nexti13->i12->nexti12 so store in necessary order
      indices.push(i13, i12)
      i13 = createVertex(modPoint, i1, i3, v1, v3, vertices, dim, axis)
      indices.push(i13, i13, i12)
      i12 = createVertex(modPoint, i1, i2, v1, v2, vertices, dim, axis)
      indices.push(i12)
      // increment
      modPoint += modulo
    }
    // add v3 triangle if necessary
    indices.push(i13, i12, i3)
    // return the remaining triangle
    return [i3, i12, i2]
  }
}

// i1 is always the vertex with an acute angle. i2 is always the furthest away from i1
// splitLeft means we start on the right side of this "1D" observation moving left
function splitLeft (
  modPoint: number,
  i1: number,
  i2: number,
  i3: number,
  v1: number,
  v2: number,
  v3: number,
  vertices: number[],
  indices: number[],
  dim: number,
  axis: number,
  modulo: number
): [number, number, number] {
  // first case is a standalone triangle
  let i12 = createVertex(modPoint, i1, i2, v1, v2, vertices, dim, axis)
  let i13 = createVertex(modPoint, i1, i3, v1, v3, vertices, dim, axis)
  indices.push(i1, i12, i13)
  modPoint -= modulo
  if (v2 > v3) {
    // create lines up to i2
    while (modPoint > v2) {
      // next triangles are i13->i12->nexti13 and nexti13->i12->nexti12 so store in necessary order
      indices.push(i13, i12)
      i13 = createVertex(modPoint, i1, i3, v1, v3, vertices, dim, axis)
      indices.push(i13, i13, i12)
      i12 = createVertex(modPoint, i1, i2, v1, v2, vertices, dim, axis)
      indices.push(i12)
      // increment
      modPoint -= modulo
    }
    // add v2 triangle if necessary
    indices.push(i13, i12, i2)
    // return the remaining triangle
    return [i13, i2, i3]
  } else {
    // create lines up to i2
    while (modPoint > v3) {
      // next triangles are i13->i12->nexti13 and nexti13->i12->nexti12 so store in necessary order
      indices.push(i13, i12)
      i13 = createVertex(modPoint, i1, i3, v1, v3, vertices, dim, axis)
      indices.push(i13, i13, i12)
      i12 = createVertex(modPoint, i1, i2, v1, v2, vertices, dim, axis)
      indices.push(i12)
      // increment
      modPoint -= modulo
    }
    // add v3 triangle if necessary
    indices.push(i13, i12, i3)
    // return the remaining triangle
    return [i3, i12, i2]
  }
}

function mod2 (x: number, n: number): number {
  return ((x % n) + n) % n
}

export function flatten (data: number[][][]): FlattenResult {
  let holeIndex = 0
  const dim = data[0][0].length
  const vertices = []
  const holeIndices = []

  for (let i = 0, pl = data.length; i < pl; i++) {
    for (let j = 0, ll = data[i].length; j < ll; j++) {
      for (let d = 0; d < dim; d++) vertices.push(data[i][j][d])
    }
    if (i > 0) {
      holeIndex += data[i - 1].length
      holeIndices.push(holeIndex)
    }
  }
  return { vertices, holeIndices, dim }
}

export function deviation (
  data: number[],
  holeIndices: number[] = [],
  dim = 2,
  triangles: number[] = []
): number {
  const hasHoles = holeIndices.length > 0
  const outerLen = hasHoles ? holeIndices[0] * dim : data.length
  let polygonArea = Math.abs(signedArea(data, 0, outerLen, dim))

  if (hasHoles) {
    for (let i = 0, len = holeIndices.length; i < len; i++) {
      const start = holeIndices[i] * dim
      const end = i < len - 1 ? holeIndices[i + 1] * dim : data.length
      polygonArea -= Math.abs(signedArea(data, start, end, dim))
    }
  }

  let trianglesArea = 0
  for (let i = 0; i < triangles.length; i += 3) {
    const a = triangles[i] * dim
    const b = triangles[i + 1] * dim
    const c = triangles[i + 2] * dim
    trianglesArea += Math.abs(
      (data[a] - data[c]) * (data[b + 1] - data[a + 1]) -
      (data[a] - data[b]) * (data[c + 1] - data[a + 1])
    )
  }

  return polygonArea === 0 && trianglesArea === 0
    ? 0
    : Math.abs((trianglesArea - polygonArea) / polygonArea)
}

function signedArea (data: number[], start: number, end: number, dim: number): number {
  let sum = 0
  for (let i = start, j = end - dim; i < end; i += dim) {
    sum += (data[j] - data[i]) * (data[i + 1] + data[j + 1])
    j = i
  }

  return sum
}
