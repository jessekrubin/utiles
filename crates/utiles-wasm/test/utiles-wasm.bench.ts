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

bench("js-quadkey", () => {
  xyz2quadkey(486, 332, 20);
});

bench("wasm-quadkey", () => {
  utw.xyz2qk(486, 332, 20);
});

bench("js-pmtile-id", () => {
  pmtiles.zxyToTileId(
    20,
    486,
    332,
  );
  // .pmtileId2(486, 332, 20)
});

bench("wasm-pmtile-id", () => {
  utw.pmtileid(486, 332, 20);
});
