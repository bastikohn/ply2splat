import test from "ava";

import { simpleFn } from "../index.js";

test("sync function from native code", (t) => {
  const r = simpleFn();
  t.is(r, 1);
});
