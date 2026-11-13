// The play page (/play/): play a library model at a chosen ply (the user, 2026-11-12). leagues/play.py is its back
// end: the model's moves come from chessai-play2 play-serve, chosen as a tournament player chooses them (the model
// calibrated on its own games, as in every arena), at an exact ply or in deep mode (iterative deepening within a think
// time, up to ply 10, optionally thinking on the user's time). chess.js decides legality and the end of the game;
// chessground (lichess.org's board) draws the board.
import { Chessground } from '/play/static/chessground.min.js';
import { Chess } from '/play/static/chess.js';

const D = JSON.parse(document.getElementById('pl-data').textContent);
const $ = (id) => document.getElementById(id);
const esc = (s) => String(s ?? '').replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' })[c]);
const ROWS = D.models || [];
const BY_KEY = new Map(ROWS.map((r) => [r.key, r]));
const PIECE = { q: 'Queen', r: 'Rook', b: 'Bishop', n: 'Knight' };
const GLYPH = { white: { q: '♕', r: '♖', b: '♗', n: '♘' }, black: { q: '♛', r: '♜', b: '♝', n: '♞' } };

const S = {
  row: null, ply: 1, you: 'white', policy: 'argmax', seed: 1,
  mode: 'exact', maxPly: 10, seconds: 30, ponder: true, job: null, nowWanted: false,  // deep mode (seconds 0: no limit)
  chess: new Chess(), moves: [], info: [],  // moves: UCI from the start; info: per ply {by, san, resp, fenBefore}
  live: false, busy: false, token: 0, over: null, takebacks: 0, started: null, startEval: null,
};
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

// ---------------------------------------------------------------- the board

const cg = Chessground($('pl-board'), {
  fen: 'start',
  orientation: 'white',
  coordinates: true,
  movable: { free: false, color: undefined, dests: new Map(), showDests: true, events: { after: onBoardMove } },
  draggable: { showGhost: true },
  premovable: { enabled: false },
  highlight: { lastMove: true, check: true },
  animation: { enabled: true, duration: 160 },
});

const turnColour = () => (S.chess.turn() === 'w' ? 'white' : 'black');

function dests() {
  const m = new Map();
  for (const mv of S.chess.moves({ verbose: true })) {
    if (!m.has(mv.from)) m.set(mv.from, []);
    if (!m.get(mv.from).includes(mv.to)) m.get(mv.from).push(mv.to);
  }
  return m;
}

function yourTurn() {
  return S.live && !S.over && !S.busy && turnColour() === S.you;
}

function syncBoard() {
  const hist = S.chess.history({ verbose: true });
  const last = hist.length ? hist[hist.length - 1] : null;
  cg.set({
    fen: S.chess.fen(),
    turnColor: turnColour(),
    lastMove: last ? [last.from, last.to] : undefined,
    check: S.chess.inCheck() ? turnColour() : false,
    movable: yourTurn() ? { color: S.you, dests: dests() } : { color: undefined, dests: new Map() },
  });
  $('pl-back').disabled = !S.live || !S.info.some((x) => x.by === 'you');
  $('pl-resign').disabled = !S.live || !!S.over;
  $('pl-typed').disabled = !yourTurn();
}

function status(html, cls = '') {
  const s = $('pl-status');
  s.className = 'pl-status' + (cls ? ' ' + cls : '');
  s.innerHTML = html;
}

// ---------------------------------------------------------------- talking to the server

async function api(name, body) {
  try {
    const r = await fetch('/play/api/' + name, {
      method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body),
    });
    const t = await r.text();
    try { return JSON.parse(t); } catch { return { ok: false, error: `the server answered ${r.status}: ${t.slice(0, 300)}` }; }
  } catch (e) {
    return { ok: false, error: 'no answer from the dashboard server (' + e + ')' };
  }
}

// ---------------------------------------------------------------- choosing the opponent

const F = { set: 'default', arch: '', kind: '', ply: '', q: '', sort: 'elo' };

function rowLabel(r) {
  const elo = r.elo != null ? ` · ladder ${Math.round(r.elo)}` : '';
  const hold = r.hold ? ' · holdout' : '';
  return `${r.archetype || '?'} · ${r.experiment} ${r.line} v${r.version} · trained at ${r.train_ply_label}${elo}${hold}`;
}

function shown() {
  const q = F.q.trim().toLowerCase();
  let rs = ROWS.filter((r) => (F.set === 'all' || r.sets.some((s) => s === 'plateaued' || s === 'holdout'))
    && (!F.arch || r.arch === F.arch) && (!F.kind || r.kind === F.kind)
    && (!F.ply || (F.ply === 'other' ? !(r.ply_num >= 1 && r.ply_num <= 4) : String(r.ply_num) === F.ply))
    && (!q || `${r.key} ${r.experiment} ${r.line} ${r.model} ${r.archetype} ${r.kind}`.toLowerCase().includes(q)));
  const by = {
    elo: (a, b) => (b.elo ?? -1e9) - (a.elo ?? -1e9) || a.key.localeCompare(b.key),
    type: (a, b) => a.type_order - b.type_order || (a.ply_num ?? 99) - (b.ply_num ?? 99) || a.key.localeCompare(b.key),
    ply: (a, b) => (a.ply_num ?? 99) - (b.ply_num ?? 99) || a.type_order - b.type_order || a.key.localeCompare(b.key),
    experiment: (a, b) => a.experiment.localeCompare(b.experiment) || a.line.localeCompare(b.line) || a.version - b.version,
  }[F.sort];
  return rs.sort(by);
}

function renderList() {
  const sel = $('pl-model');
  const keep = sel.value;
  const rs = shown();
  sel.innerHTML = rs.map((r) => `<option value="${esc(r.key)}">${esc(rowLabel(r))}</option>`).join('');
  if (rs.some((r) => r.key === keep)) sel.value = keep;
  $('pl-count').textContent = `${rs.length} of ${ROWS.length} models`;
  if (!sel.value && rs.length) sel.value = rs[0].key;
  showRow();
}

function showRow() {
  const r = BY_KEY.get($('pl-model').value);
  const box = $('pl-row');
  if (!r) { box.innerHTML = '<span class="muted">No model matches the filters.</span>'; return; }
  const bits = [
    ['Model', `${r.model || r.archetype} (${r.kind})`],
    ['Run', `${r.experiment}, line ${r.line}, version ${r.version}${r.role ? ` (${r.role})` : ''}`],
    ['Trained', `${r.train_ply_label}; ${r.regime}; ended by ${r.end}${r.field ? `; field ${r.field}` : ''}`],
    ['Ladder', r.elo != null ? `${r.elo.toFixed(1)} at its training ply (the latest global ladder run)` : 'not on the global ladder'],
  ];
  if (r.hold) bits.push(['Holdout', r.hold]);
  box.innerHTML = bits.map(([k, v]) => `<div><span class="pl-k">${esc(k)}</span> ${esc(v)}</div>`).join('');
  if (!S.live || S.over) $('pl-ply').value = String(r.default_ply);
}

for (const [id, key] of [['pl-f-set', 'set'], ['pl-f-arch', 'arch'], ['pl-f-kind', 'kind'], ['pl-f-ply', 'ply'], ['pl-sort', 'sort']]) {
  $(id).addEventListener('change', (e) => { F[key] = e.target.value; renderList(); });
}
$('pl-q').addEventListener('input', (e) => { F.q = e.target.value; renderList(); });
$('pl-model').addEventListener('change', showRow);

function syncMode() {
  const deep = radio('pl-mode') === 'deep';
  $('pl-deep-opts').hidden = !deep;
  $('pl-ply-l').hidden = deep;
  const sm = document.querySelector('input[name="pl-policy"][value="softmax"]');
  sm.disabled = deep;
  if (deep && sm.checked) document.querySelector('input[name="pl-policy"][value="argmax"]').checked = true;
}
document.querySelectorAll('input[name="pl-mode"]').forEach((x) => x.addEventListener('change', syncMode));

// ---------------------------------------------------------------- a game

function radio(name) {
  const x = document.querySelector(`input[name="${name}"]:checked`);
  return x ? x.value : null;
}

async function newGame() {
  const r = BY_KEY.get($('pl-model').value);
  if (!r) { status('Choose a model first.', 'bad'); return; }
  stopEngine();
  const tok = ++S.token;
  const col = radio('pl-colour') || 'white';
  const deep = radio('pl-mode') === 'deep';
  Object.assign(S, {
    row: r, ply: +$('pl-ply').value, policy: deep ? 'argmax' : radio('pl-policy') || 'argmax',
    mode: deep ? 'deep' : 'exact', maxPly: +$('pl-maxply').value, seconds: +$('pl-think').value,
    ponder: deep && $('pl-ponder').checked, job: null,
    you: col === 'random' ? (Math.random() < 0.5 ? 'white' : 'black') : col,
    seed: 1 + Math.floor(Math.random() * 999999999),
    chess: new Chess(), moves: [], info: [], live: false, busy: true, over: null, takebacks: 0, started: null,
    startEval: null,
  });
  cg.set({ orientation: S.you });
  syncBoard();
  renderMoves();
  renderEval();
  $('pl-ponder-st').textContent = '';
  $('pl-who').innerHTML = `You play <b>${S.you}</b> against <b>${esc(rowLabel(r))}</b> `
    + (S.mode === 'deep'
      ? `in deep mode: up to ply <b>${S.maxPly}</b>, <b>${thinkText(S.seconds)}</b> a move${S.ponder ? ', thinking on your time' : ''}, its best move each time`
      : `at ply <b>${S.ply}</b>, ` + (S.policy === 'argmax' ? 'its best move each time (the log-odds argmax)' : 'sampling as the arenas do (log-odds softmax, T 0.02)'))
    + `. Seed ${S.seed}.`;
  status('Preparing the model: its own calibration games come from the arenas\' cache, or are made once here '
    + '(600 games, usually seconds), then it loads…');
  const p = await api('prepare', { key: r.key });
  if (tok !== S.token) return;
  S.busy = false;
  if (!p.ok) { status('Could not prepare the model: ' + esc(p.error), 'bad'); syncBoard(); return; }
  S.live = true;
  S.started = new Date().toISOString();
  S.startEval = p.probs || null;
  $('pl-cal').innerHTML = `Calibration: ${esc(p.calibration || '?')} · own games ${esc(p.own_source)}`
    + (p.load_ms ? ` · loaded in ${Math.round(p.load_ms)} ms` : '');
  renderEval();
  if (S.you === 'black') engineMove();
  else { syncBoard(); status('Your move.'); }
}

function onBoardMove(orig, dest) {
  if (!yourTurn()) { syncBoard(); return; }
  const cands = S.chess.moves({ verbose: true }).filter((m) => m.from === orig && m.to === dest);
  if (!cands.length) { syncBoard(); return; }
  if (cands.some((m) => m.promotion)) askPromotion(dest).then((p) => (p ? playYours(orig, dest, p) : syncBoard()));
  else playYours(orig, dest);
}

function askPromotion(dest) {
  return new Promise((resolve) => {
    const box = $('pl-promo');
    const colour = S.you;
    box.innerHTML = '<div class="pl-promo-in" role="dialog" aria-label="promote to"><span>Promote to</span>'
      + ['q', 'r', 'b', 'n'].map((p) => `<button type="button" data-p="${p}" class="pl-promo-b">`
        + `<span class="pl-glyph" aria-hidden="true">${GLYPH[colour][p]}</span>${PIECE[p]}</button>`).join('')
      + '<button type="button" data-p="" class="pl-btn">Cancel</button></div>';
    box.hidden = false;
    const done = (p) => { box.hidden = true; box.innerHTML = ''; box.onkeydown = null; resolve(p); };
    box.querySelectorAll('button').forEach((b) => b.addEventListener('click', () => done(b.dataset.p || null)));
    box.onkeydown = (e) => { if (e.key === 'Escape') { e.preventDefault(); done(null); } };
    box.querySelector('button').focus();
  });
}

function playYours(from, to, promotion) {
  let m;
  try { m = S.chess.move({ from, to, promotion }); } catch { syncBoard(); return; }
  S.moves.push(m.from + m.to + (m.promotion || ''));
  S.info.push({ by: 'you', san: m.san });
  syncBoard();
  renderMoves();
  if (checkOver()) return;
  engineMove();
}

async function engineMove() {
  $('pl-ponder-st').textContent = '';
  if (S.mode === 'deep') return deepMove();
  const tok = S.token;
  S.busy = true;
  syncBoard();
  status(`The model is thinking (ply ${S.ply})…`);
  const fenBefore = S.chess.fen();
  const a = await api('move', { key: S.row.key, ply: S.ply, policy: S.policy, seed: S.seed, moves: S.moves });
  if (tok !== S.token) return;
  S.busy = false;
  if (!a.ok) { status('The engine failed: ' + esc(a.error) + ' — take back or start a new game.', 'bad'); syncBoard(); return; }
  playEngine(a, fenBefore);
}

// ---------------------------------------------------------------- deep mode

const thinkText = (s) => (+s <= 0 ? 'no limit' : +s >= 120 ? `${+s / 60} min` : `${+s} s`);
const fmtS = (ms) => (ms < 10000 ? (ms / 1000).toFixed(1) : Math.round(ms / 1000)) + ' s';
const STOPS = {
  'max depth': 'the max ply done', time: 'the think time was up',
  'time (the next iteration would not end in it)': 'the next ply would not have finished in the think time',
  stopped: 'stopped (Move now)', 'mate found': 'a mate found', 'one legal move': 'its only legal move',
};
const stopText = (w) => STOPS[w] || w;

function stopEngine() {
  // A running deep search (it answers with its deepest finished ply; the page ignores it) and the ponder.
  if (S.job) api('stop', { job: S.job, ponder: true });
  else if (S.mode === 'deep') api('stop', { ponder: true });
  S.job = null;
  $('pl-now').hidden = true;
}

function thinking(t0, last) {
  const el = Date.now() - t0;
  const lim = S.seconds > 0 ? ` of ${thinkText(S.seconds)}` : ' (no limit)';
  const where = last ? `ply ${last.depth} done: ${esc(last.san)} ${fmtVal(last.value)}` : 'ply 1…';
  status(`<span class="pl-spin" aria-hidden="true"></span> The model is thinking: ${fmtS(el)}${lim} · ${where}`
    + (S.maxPly ? ` · max ply ${S.maxPly}` : ''));
}

async function deepMove() {
  const tok = S.token;
  S.busy = true;
  syncBoard();
  const fenBefore = S.chess.fen();
  const fresh = !S.info.some((x) => x.by === 'model');  // a new game: a new table
  const t0 = Date.now();
  thinking(t0, null);
  S.nowWanted = false;
  $('pl-now').hidden = false;
  $('pl-now').disabled = false;
  const st = await api('deep', { key: S.row.key, max_ply: S.maxPly, seconds: S.seconds, seed: S.seed, ponder: S.ponder,
    fresh, moves: S.moves });
  if (tok !== S.token) { if (st.ok) api('stop', { job: st.job }); return; }
  if (!st.ok) { S.busy = false; $('pl-now').hidden = true; status('The engine failed: ' + esc(st.error) + ' — take back or start a new game.', 'bad'); syncBoard(); return; }
  S.job = st.job;
  if (S.nowWanted) api('stop', { job: st.job });  // Move now, pressed before the search had its id
  let last = null;
  const timer = setInterval(() => { if (tok === S.token) thinking(t0, last); }, 250);
  try {
    for (;;) {
      await sleep(400);
      if (tok !== S.token) return;
      const p = await api('poll', { job: st.job });
      if (tok !== S.token) return;
      if (!p.ok) { S.busy = false; status('Lost the search: ' + esc(p.error) + ' — take back or start a new game.', 'bad'); syncBoard(); return; }
      if (p.last) last = p.last;
      if (p.state === 'running') continue;
      S.busy = false;
      const a = p.result;
      if (p.state !== 'done' || !a || !a.ok) { status('The engine failed: ' + esc(p.error || (a && a.error)) + ' — take back or start a new game.', 'bad'); syncBoard(); return; }
      playEngine(a, fenBefore);
      if (S.ponder && !S.over) ponderWatch(S.token);
      return;
    }
  } finally {
    clearInterval(timer);
    if (S.job === st.job) { S.job = null; $('pl-now').hidden = true; }
  }
}

async function ponderWatch(tok) {
  // While it is your turn: what the ponder has done (the engine's latest @@ponder line, for this position).
  const box = $('pl-ponder-st');
  while (tok === S.token && yourTurn()) {
    const p = await api('ponder', {});
    if (tok !== S.token || !yourTurn()) break;
    const d = p.ok && p.ponder;
    if (d && d.plies === S.moves.length + 1) {
      const doing = d.state === 'end' ? `stopped at ply ${d.depth} (${stopText(d.stop)})`
        : d.state === 'iteration' ? `ply ${d.depth} done, it would answer ${esc(d.san)}` : 'started';
      box.innerHTML = `<span class="pl-spin${d.state === 'end' ? ' off' : ''}" aria-hidden="true"></span> Thinking on your time, `
        + `expecting ${esc(d.predicted_san)}: ${doing}.`;
      if (d.state === 'end') break;
    }
    await sleep(1000);
  }
}

function playEngine(a, fenBefore) {
  const u = a.move;
  let m;
  try { m = S.chess.move({ from: u.slice(0, 2), to: u.slice(2, 4), promotion: u[4] }); } catch (e) {
    status(`The engine's move ${esc(u)} is illegal by chess.js's rules (${esc(e)}).`, 'bad');
    syncBoard();
    return;
  }
  S.moves.push(u);
  S.info.push({ by: 'model', san: m.san, resp: a, fenBefore });
  syncBoard();
  renderMoves();
  renderEval();
  if (checkOver()) return;
  let note = '';
  if (m.san !== a.san) note += ` (the engine wrote ${esc(a.san)})`;
  if (a.end_after) note += ` By the engine's own rules the game is over here (${esc(a.end_after.reason)}), by chess.js's it isn't; play goes on.`;
  status('Your move.' + note);
}

function checkOver() {
  const c = S.chess;
  let res = null;
  let why = null;
  if (c.isCheckmate()) { why = 'checkmate'; res = c.turn() === 'w' ? '0-1' : '1-0'; }
  else if (c.isStalemate()) { why = 'stalemate'; res = '1/2-1/2'; }
  else if (c.isInsufficientMaterial()) { why = 'insufficient material'; res = '1/2-1/2'; }
  else if (c.isThreefoldRepetition()) { why = 'threefold repetition'; res = '1/2-1/2'; }
  else if (c.isDrawByFiftyMoves()) { why = 'fifty-move rule'; res = '1/2-1/2'; }
  if (!res) return false;
  finish(res, why);
  return true;
}

function resultText(res, why) {
  const youWon = (res === '1-0' && S.you === 'white') || (res === '0-1' && S.you === 'black');
  if (res === '1/2-1/2') return `Draw (${why}).`;
  return (youWon ? 'You won' : 'The model won') + ` (${why}): ${res}.`;
}

async function finish(res, why) {
  if (S.mode === 'deep') { stopEngine(); $('pl-ponder-st').textContent = ''; }
  S.over = { res, why };
  syncBoard();
  renderMoves();
  status(resultText(res, why) + ' Saving the game…', res === '1/2-1/2' ? '' : 'ok');
  const tok = S.token;
  const deep = {};
  S.info.forEach((x, i) => {
    const a = x.resp;
    if (x.by === 'model' && a && a.op === 'deep') {
      const p = ponderOf(a);
      deep[i] = { depth: a.depth, ms: a.think_ms, stop: a.stop, ponder: p ? (p.hit ? 'hit' : 'miss') : null,
        pdepth: p && p.hit ? p.depth : 0 };
    }
  });
  const a = await api('save', {
    key: S.row.key,
    game: { moves: S.moves, result: res, termination: why, colour: S.you, ply: S.ply, policy: S.policy,
            temperature: 0.02, seed: S.seed, takebacks: S.takebacks, started: S.started,
            mode: S.mode, max_ply: S.maxPly, seconds: S.seconds, ponder: S.ponder, deep },
  });
  if (tok !== S.token) return;
  if (!a.ok) { status(resultText(res, why) + ' Not saved: ' + esc(a.error), 'bad'); return; }
  status(resultText(res, why) + ` Saved as <a href="/play/games/${encodeURIComponent(a.file)}">${esc(a.file)}</a>.`);
  addSaved(a.entry);
}

function takeBack() {
  if (!S.live || !S.info.some((x) => x.by === 'you')) return;
  stopEngine();
  $('pl-ponder-st').textContent = '';
  S.token++;
  S.busy = false;
  S.over = null;
  while (S.info.length) {
    const last = S.info.pop();
    S.moves.pop();
    S.chess.undo();
    if (last.by === 'you') break;
  }
  S.takebacks++;
  syncBoard();
  renderMoves();
  renderEval();
  status(`Taken back (${S.takebacks} so far). Your move.`);
}

function resign() {
  if (!S.live || S.over) return;
  stopEngine();
  $('pl-ponder-st').textContent = '';
  S.token++;
  S.busy = false;
  finish(S.you === 'white' ? '0-1' : '1-0', 'resignation');
}

$('pl-new').addEventListener('click', newGame);
$('pl-new2').addEventListener('click', newGame);
$('pl-back').addEventListener('click', takeBack);
$('pl-flip').addEventListener('click', () => cg.toggleOrientation());
$('pl-resign').addEventListener('click', resign);
$('pl-now').addEventListener('click', () => {
  if (!S.busy || S.mode !== 'deep') return;
  if (S.job) api('stop', { job: S.job });
  else S.nowWanted = true;
  $('pl-now').disabled = true;
});
window.addEventListener('pagehide', () => {
  // Leaving the page stops its search and the ponder (the server also stops a search nobody polls).
  if (S.mode !== 'deep') return;
  const body = JSON.stringify(S.job ? { job: S.job, ponder: true } : { ponder: true });
  navigator.sendBeacon('/play/api/stop', new Blob([body], { type: 'application/json' }));
});
$('pl-typed').addEventListener('keydown', (e) => {
  if (e.key !== 'Enter') return;
  const t = e.target.value.trim();
  if (!t || !yourTurn()) return;
  const legal = S.chess.moves({ verbose: true });
  const m = legal.find((x) => x.san === t || x.san.replace(/[+#]$/, '') === t.replace(/[+#]$/, '')
    || x.from + x.to + (x.promotion || '') === t.toLowerCase()
    || (!x.promotion ? false : x.from + x.to === t.toLowerCase() && x.promotion === 'q'));
  if (!m) { status(`${esc(t)} is not a legal move here (type SAN like Nf3 or UCI like g1f3).`, 'bad'); return; }
  e.target.value = '';
  playYours(m.from, m.to, m.promotion);
});

// ---------------------------------------------------------------- what the page shows

function renderMoves() {
  const out = [];
  for (let i = 0; i < S.info.length; i += 2) {
    const w = S.info[i];
    const b = S.info[i + 1];
    const cell = (x) => (x ? `<td class="${x.by === 'model' ? 'pl-mm' : ''}"${x.resp ? ` data-tip="${esc(tipOf(x))}"` : ''}>`
      + `${esc(x.san)}${x.resp ? ` <span class="pl-ms">${msNote(x.resp)}</span>` : ''}</td>` : '<td></td>');
    out.push(`<tr><td class="pl-n">${i / 2 + 1}.</td>${cell(w)}${cell(b)}</tr>`);
  }
  if (S.over) out.push(`<tr><td></td><td colspan="2" class="pl-res">${esc(S.over.res)} <span class="muted">${esc(S.over.why)}</span></td></tr>`);
  $('pl-moves').innerHTML = out.join('') || '<tr><td class="muted">No moves yet.</td></tr>';
  const box = $('pl-moves-box');
  box.scrollTop = box.scrollHeight;
}

const fmtMs = (ms) => (ms < 10 ? ms.toFixed(1) : Math.round(ms)) + ' ms';
const msNote = (a) => (a.op === 'deep'
  ? `d${a.depth} · ${a.think_ms < 1000 ? fmtMs(a.think_ms) : fmtS(a.think_ms)}${ponderOf(a) && ponderOf(a).hit ? ' · pondered' : ''}`
  : fmtMs(a.think_ms));
const num = (v) => (typeof v === 'string' ? (v.includes('inf') ? (v[0] === '-' ? -Infinity : Infinity) : NaN) : v);

function fmtVal(v) {
  v = num(v);
  if (v === Infinity) return 'wins';
  if (v === -Infinity) return 'loses';
  return (v > 0 ? '+' : v < 0 ? '−' : '') + Math.abs(v).toFixed(2);
}

// The ponder that this move could use: none after a take-back or in a new game (the engine cleared its table).
const ponderOf = (a) => (a.ponder && !(a.tt && a.tt.cleared) ? a.ponder : null);

function ponderNote(a) {
  const p = ponderOf(a);
  if (!p) return '';
  return p.hit
    ? `; it had pondered your reply ${p.predicted_san} to ply ${p.depth} on your time`
    : `; it had expected ${p.predicted_san} (pondered to ply ${p.depth}), so it searched afresh`;
}

function tipOf(x) {
  const a = x.resp;
  if (a.op === 'deep') {
    return `the model's move ${x.san}: deep search to ply ${a.depth} (max ${a.max_depth}) in ${fmtS(a.think_ms)}, `
      + `${stopText(a.stop)}${ponderNote(a)}; value ${fmtVal(a.value)}`;
  }
  return `the model's move ${x.san}: searched ${S.ply} plies in ${fmtMs(a.think_ms)}`
    + (a.value !== undefined ? `, value ${fmtVal(a.value)} (ln P(win) − ln P(loss) for the side that moved)` : '');
}

function bar(p, title) {
  const w = (100 * p.white).toFixed(0);
  const d = (100 * p.draw).toFixed(0);
  const b = (100 * p.black).toFixed(0);
  return `<div class="pl-evrow"><span class="pl-k">${esc(title)}</span><div class="pl-bar" role="img" `
    + `aria-label="White wins ${w}%, draw ${d}%, Black wins ${b}%"><span class="w" style="flex:${p.white}"></span>`
    + `<span class="d" style="flex:${p.draw}"></span><span class="b" style="flex:${p.black}"></span></div>`
    + `<span class="pl-pct">White ${w}% · draw ${d}% · Black ${b}%</span></div>`;
}

function renderEval() {
  const box = $('pl-eval');
  const lastModel = [...S.info].reverse().find((x) => x.by === 'model');
  if (!S.live && !lastModel) { box.innerHTML = '<span class="muted">The model\'s view appears once the game starts.</span>'; return; }
  const parts = [];
  if (!lastModel) {
    if (S.startEval) parts.push(bar(S.startEval, 'Its view of the start'));
  } else {
    const a = lastModel.resp;
    const side = new Chess(lastModel.fenBefore).turn() === 'w' ? 'White' : 'Black';
    parts.push(bar(a.probs_before, 'Before its move'));
    parts.push(bar(a.probs_after, 'After its move'));
    let alt = '';
    if (a.values) {
      const c = new Chess(lastModel.fenBefore);
      const top = Object.entries(a.values).map(([u, v]) => [u, num(v)]).sort((x, y) => y[1] - x[1]).slice(0, 4);
      alt = top.map(([u, v]) => {
        let san = u;
        try { san = new Chess(lastModel.fenBefore).move({ from: u.slice(0, 2), to: u.slice(2, 4), promotion: u[4] }).san; } catch { /* keep UCI */ }
        return `${esc(san)} ${fmtVal(v)}`;
      }).join(' · ');
      void c;
    }
    if (a.op === 'deep') {
      const its = (a.iters || []).map((it) => `${it.depth}: ${esc(it.san)}${it.pondered ? '*' : ''}`).join(' · ');
      parts.push(`<div class="pl-search">Its deep search reached <b>ply ${a.depth}</b> of ${a.max_depth} in `
        + `<b>${fmtS(a.think_ms)}</b> (${esc(stopText(a.stop))}${esc(ponderNote(a))}), and valued <b>${esc(lastModel.san)}</b> at `
        + `<b>${fmtVal(a.value)}</b> for ${side} (ln P(win) − ln P(loss)), over ${a.legal} legal moves, `
        + `${(a.nodes / 1e6).toFixed(a.nodes < 1e6 ? 2 : 1)} M positions.`
        + `<br><span class="muted">Its move at each ply (* pondered): ${its}</span>`
        + (alt ? `<br><span class="muted">Moves valued exactly at ply ${a.depth}: ${alt}</span>` : '') + '</div>');
    } else {
      parts.push(`<div class="pl-search">Its search (${S.ply} ${S.ply === 1 ? 'ply' : 'plies'}) valued <b>${esc(lastModel.san)}</b> at `
        + `<b>${fmtVal(a.value)}</b> for ${side} (ln P(win) − ln P(loss), the value it maximises) and took `
        + `${fmtMs(a.think_ms)}${a.legal ? ` over ${a.legal} legal moves` : ''}.${alt ? `<br><span class="muted">Its best-valued moves: ${alt}</span>` : ''}</div>`);
    }
  }
  box.innerHTML = parts.join('');
}

function addSaved(e) {
  if (!e) return;
  D.saved = [e, ...(D.saved || [])].slice(0, 40);
  renderSaved();
}

function renderSaved() {
  const rs = D.saved || [];
  $('pl-saved').innerHTML = rs.length ? rs.map((g) => `<li><a href="/play/games/${encodeURIComponent(g.file)}">${esc(g.date || '')} `
    + `${esc(g.time || '')}</a> ${esc(g.white)} – ${esc(g.black)} <b>${esc(g.result)}</b> `
    + `<span class="muted">${esc(g.termination || '')}, ${esc(g.plies)} plies${+g.takebacks ? `, ${esc(g.takebacks)} take-backs` : ''}</span></li>`).join('')
    : '<li class="muted">None yet.</li>';
}

renderList();
syncMode();
renderMoves();
renderEval();
renderSaved();
syncBoard();
status('Choose an opponent and a ply, then press New game.');
window.__play = S;  // the game's state, for checks from the browser's console (and the page's UI test)
