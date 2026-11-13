# play-static: the play page's browser files

The dashboard serves these at `/play/static/<name>` for its play page (`/play/`, leagues/play.py). Nothing loads from a
CDN. The server serves only the files listed in play.py's `STATIC_FILES`.

## Vendored (2026-11-12, from the npm registry, unchanged)

| File | From | Version | License |
|---|---|---|---|
| chessground.min.js | chessground (lichess.org's board), `dist/chessground.min.js` | 9.2.1 | GPL-3.0-or-later, LICENSE-chessground |
| chessground.base.css, chessground.brown.css, chessground.cburnett.css | chessground, `assets/` (board, brown squares, cburnett pieces as embedded SVG) | 9.2.1 | GPL-3.0-or-later, LICENSE-chessground |
| chess.js | chess.js, `dist/esm/chess.js` (move generation, legality, game end, SAN) | 1.4.0 | BSD-2-Clause, LICENSE-chess.js |

The tarballs, with sha256:
- `https://registry.npmjs.org/chessground/-/chessground-9.2.1.tgz` 6bcb9f102c05aa1d5e3cfe9eb82338fb79d829cd575b545a6896706b4a06d058
- `https://registry.npmjs.org/chess.js/-/chess.js-1.4.0.tgz` dce92e280439d7a16f645be94f22e6da50e579a1f3011d0a2381755bfe2c2ef7

The files' sha256:

| File | sha256 |
|---|---|
| chessground.min.js | 915457e1a6b84abd9778d3ea5317a9fccaf59f4caa78196efbd92f586c0f0e9b |
| chessground.base.css | 77cbfd65f9b3f64f310a0b8eab0e34ef9f364172e4f678fc05c114b385a341a6 |
| chessground.brown.css | 80de613fe928a947e88b983835286f21c723738d9319b1f8868442bb56fb1577 |
| chessground.cburnett.css | f9216192b5383a83e62a81de1cf58520f7c251f6f24930f5710a3c797f78ced4 |
| chess.js | 76c7c34f0e2e9ab076521a5d6fe786a9cce537bb1b6f29d32a9c9970b5b232d2 |

chessground is GPL-3.0. The page is a local tool on 127.0.0.1 and is not distributed. If it ever is, the GPL's terms
apply to what is shipped with it.

## Ours

- `play.js`: the page's script, an ES module that imports the two above.
- `play.css`: the page's styles, on the dashboard's colour variables.
