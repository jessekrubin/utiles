import {expect, test} from 'vitest';

import * as ut from '../index.js';

test('tile', () => {
    let t = new ut.Tile(0, 0, 0);
    expect(t).toBeDefined();
    expect (t.x).toBe(0);
    expect (t.y).toBe(0);
    expect (t.z).toBe(0);
});

