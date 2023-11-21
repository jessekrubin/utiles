import {expect, test} from 'vitest';

import * as ut from '../index';

test('dev', () => {
  expect(true).toBe(true);
});

test('cli', () => {
    expect(ut).toBeDefined();
    console.log(ut);
    // the block below nukes the other stdout from test...
    // let r = ut.utCli(['-h']);
    // console.log(r);
    // expect(
    //     r
    // ).toBe(0);
})
