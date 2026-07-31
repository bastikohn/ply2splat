import test from "ava";

import { convert, getSplatCount } from "../index.js";

const fixture = Buffer.from(`ply
format ascii 1.0
element vertex 2
property float x
property float y
property float z
property float f_dc_0
property float f_dc_1
property float f_dc_2
property float opacity
property float scale_0
property float scale_1
property float scale_2
property float rot_0
property float rot_1
property float rot_2
property float rot_3
end_header
0.0 0.0 0.0 0.5 0.5 0.5 1.0 0.1 0.1 0.1 1.0 0.0 0.0 0.0
1.0 1.0 1.0 0.1 0.1 0.1 0.5 0.2 0.2 0.2 0.0 1.0 0.0 0.0
`);

test("converts PLY data to SPLAT bytes", (t) => {
  const result = convert(fixture);

  t.is(result.count, 2);
  t.is(result.data.length, 64);
  t.is(getSplatCount(result.data), 2);
});
