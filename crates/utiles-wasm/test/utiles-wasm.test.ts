import * as pmtiles from "pmtiles";
import { bench, expect, test } from "vitest";
import * as utw from "../pkg/utiles_wasm.js";

function xyz2quadkey(
  x: number,
  y: number,
  z: number,
) {
  let quadkey = "";
  for (let i = z; i > 0; i--) {
    let digit = 0;
    const mask = 1 << (i - 1);
    if ((x & mask) !== 0) digit += 1;
    if ((y & mask) !== 0) digit += 2;
    quadkey += digit;
  }
  return quadkey;
}
test(
  "uno",
  () => {
    console.log("uno");
    expect(1).toBe(1);
  },
);
// 486, 332, 20
test(
  "thingy",
  async () => {
    console.log("thingy");
    console.log(utw);
    const added = utw.add(1, 2);
    expect(added).toBe(3);

    // const called = await utw.default()
    // console.log(called)

    // expect(utw.add(1, 2)).toBe(3)
  },
);

test(
  "quadkey",
  async () => {
    const jsres = xyz2quadkey(486, 332, 20);
    const wasmres = utw.xyz2qk(486, 332, 20);
    expect(jsres).toBe(wasmres);

    // const called = await utw.default()
    // console.log(called)

    // expect(utw.add(1, 2)).toBe(3)
  },
);

test(
  "pmtileid",
  async () => {
    const jsres = pmtiles.zxyToTileId(
      15,
      486,
      332,
    );
    const wasmres = utw.pmtileid(486, 332, 15);

    console.log(
      {
        jsres,
        wasmres,
      },
    );
    expect(jsres).toBe(
      // debigint
      Number(wasmres),
    );
  },
);
