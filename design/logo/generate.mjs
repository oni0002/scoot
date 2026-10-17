// Generates the Scoot logo and every icon derived from it.
//   node design/logo/generate.mjs
// Writes the SVG masters next to this file, then the app icons (src-tauri/icons) and the tray
// icons (src-tauri/icons/tray), plus the path data the UI draws (src/logo.ts). Rasterising goes
// through `pnpm tauri icon`, so no extra deps.
//
// Geometry is analytic: two closed paths, no masks. In the local frame the hub sits at the
// origin and the stem points along +x; the output is rotated so the stem points up-right.
// The ring's inner edge is the mint shape offset by `gap`, so the gap is constant everywhere
// (fillets included) and R = hubR + gap + T.
import { execFileSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const INK = '#1F2326';
const MINT = '#5FE3A8';

// 1024 units. Used for 48px and up.
const LARGE = {
  R: 440, // wheel outer radius
  T: 134, // ring thickness
  gap: 48, // clear space around hub and stem
  W: 236, // stem width
  rc: 58, // stem tip corner radius
  out: 96, // how far the stem tip passes the wheel's outer edge, along its axis
  f: 72, // fillet where stem meets hub; the ring side gets f - gap
  re: 14, // rounding on the ring ends next to the stem
  angle: 45,
  optical: 0.5, // share of the ink centroid's offset to compensate (0 = geometric centre)
};

// 32px and below: bigger wheel, wider gap and ring so the gap survives at 16px.
const SMALL = { ...LARGE, R: 476, T: 150, gap: 84, f: 96, out: 60, rc: 60, re: 0, optical: 0 };

const SIZE = 1024;
const n = (v) => +v.toFixed(2);

function geometry(p) {
  const hubR = p.R - p.T - p.gap;
  const h = p.W / 2;
  const fy = h + p.f;
  const fx = Math.sqrt((hubR + p.f) ** 2 - fy ** 2); // stem/hub fillet centre (fx, ±fy)
  const ey = h + p.gap + p.re;
  const ex = Math.sqrt((p.R - p.re) ** 2 - ey ** 2); // ring-end fillet centre (ex, ±ey)
  return { ...p, hubR, h, L: p.R + p.out, fx, fy, ex, ey };
}

function pathBuilder(g, cx, cy) {
  const a = (-g.angle * Math.PI) / 180;
  const c = Math.cos(a);
  const s = Math.sin(a);
  const P = ([x, y]) => `${n(cx + x * c - y * s)} ${n(cy + x * s + y * c)}`;
  const d = [];
  return {
    M: (pt) => d.push(`M${P(pt)}`),
    L: (pt) => d.push(`L${P(pt)}`),
    A: (r, large, sweep, pt) => d.push(`A${n(r)} ${n(r)} 0 ${large} ${sweep} ${P(pt)}`),
    done: () => `${d.join('')}Z`,
  };
}

// point on the hub-side circle of radius r, towards the fillet centre
const onHub = (g, r) => [(g.fx * r) / (g.hubR + g.f), (g.fy * r) / (g.hubR + g.f)];

function mintPath(g, cx, cy) {
  const { hubR, h, L, fx, f, rc } = g;
  const [tx, ty] = onHub(g, hubR);
  const b = pathBuilder(g, cx, cy);
  b.M([tx, -ty]);
  b.A(hubR, 1, 0, [tx, ty]);
  b.A(f, 0, 1, [fx, h]);
  b.L([L - rc, h]);
  b.A(rc, 0, 0, [L, h - rc]);
  b.L([L, -h + rc]);
  b.A(rc, 0, 0, [L - rc, -h]);
  b.L([fx, -h]);
  b.A(f, 0, 1, [tx, -ty]);
  return b.done();
}

function ringPath(g, cx, cy) {
  const { R, hubR, h, gap, fx, f, re, ex, ey } = g;
  const [ox, oy] = [(ex * R) / (R - re), (ey * R) / (R - re)];
  const [ix, iy] = onHub(g, hubR + gap);
  const ri = f - gap;
  const b = pathBuilder(g, cx, cy);
  b.M([ex, -(h + gap)]);
  if (re > 0) b.A(re, 0, 0, [ox, -oy]);
  b.A(R, 1, 0, [ox, oy]);
  if (re > 0) b.A(re, 0, 0, [ex, h + gap]);
  b.L([fx, h + gap]);
  if (ri > 0) b.A(ri, 0, 0, [ix, iy]);
  b.A(hubR + gap, 1, 1, [ix, -iy]);
  if (ri > 0) b.A(ri, 0, 0, [fx, -(h + gap)]);
  return b.done();
}

// Ink centroid relative to the wheel centre, sampled on a grid (corner rounding ignored).
function centroid(g) {
  const [c, s] = [Math.cos((g.angle * Math.PI) / 180), Math.sin((g.angle * Math.PI) / 180)];
  let sx = 0;
  let sy = 0;
  let count = 0;
  for (let y = -SIZE / 2; y < SIZE / 2; y += 4)
    for (let x = -SIZE / 2; x < SIZE / 2; x += 4) {
      const lx = x * c - y * s; // canvas -> local (undo the -angle rotation)
      const ly = x * s + y * c;
      const r = Math.hypot(lx, ly);
      const mint = r < g.hubR || (lx >= 0 && lx <= g.L && Math.abs(ly) <= g.h);
      const cut = r < g.hubR + g.gap || (lx >= 0 && Math.abs(ly) <= g.h + g.gap);
      if (mint || (r <= g.R && !cut)) {
        sx += x;
        sy += y;
        count++;
      }
    }
  return [sx / count, sy / count];
}

function paths(p) {
  const g = geometry(p);
  const [mx, my] = centroid(g);
  const cx = SIZE / 2 - mx * g.optical;
  const cy = SIZE / 2 - my * g.optical;
  return { ring: ringPath(g, cx, cy), mint: mintPath(g, cx, cy) };
}

function svg(p, ring, mint) {
  const d = paths(p);
  return (
    `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${SIZE} ${SIZE}">` +
    `<path fill="${ring}" d="${d.ring}"/>` +
    `<path fill="${mint}" d="${d.mint}"/></svg>\n`
  );
}

// PNG-compressed ICO (Windows Vista+)
function ico(pngs) {
  const header = Buffer.alloc(6 + 16 * pngs.length);
  header.writeUInt16LE(1, 2);
  header.writeUInt16LE(pngs.length, 4);
  let offset = header.length;
  pngs.forEach(({ size, data }, i) => {
    const e = 6 + 16 * i;
    header.writeUInt8(size >= 256 ? 0 : size, e);
    header.writeUInt8(size >= 256 ? 0 : size, e + 1);
    header.writeUInt16LE(1, e + 4);
    header.writeUInt16LE(32, e + 6);
    header.writeUInt32LE(data.length, e + 8);
    header.writeUInt32LE(offset, e + 12);
    offset += data.length;
  });
  return Buffer.concat([header, ...pngs.map((p) => p.data)]);
}

const here = dirname(fileURLToPath(import.meta.url));
const root = join(here, '..', '..');
const icons = join(root, 'src-tauri', 'icons');

const tauriIcon = (args) =>
  execFileSync('pnpm', ['tauri', 'icon', ...args], { cwd: root, stdio: 'inherit', shell: true });

// SVG masters
const masters = {
  'icon.svg': svg(LARGE, INK, MINT),
  'icon-small.svg': svg(SMALL, INK, MINT),
  'tray-light.svg': svg(SMALL, INK, INK), // light taskbar
  'tray-dark.svg': svg(SMALL, '#FFFFFF', '#FFFFFF'), // dark taskbar
};
for (const [name, text] of Object.entries(masters)) writeFileSync(join(here, name), text);

// Path data for the UI, which draws the mark in a single colour
const ui = paths(LARGE);
writeFileSync(
  join(root, 'src', 'logo.ts'),
  `// Generated by design/logo/generate.mjs. Do not edit.
` +
    `export const LOGO_VIEWBOX = '0 0 ${SIZE} ${SIZE}';
` +
    `export const LOGO_PATHS = ['${ui.ring}', '${ui.mint}'];
`,
);
execFileSync('pnpm', ['prettier', '--write', 'src/logo.ts'], { cwd: root, stdio: 'inherit', shell: true });

// App icons: the full set from the large master, then the small sizes from the small master
tauriIcon([join(here, 'icon.svg'), '-o', icons]);
const tmp = mkdtempSync(join(tmpdir(), 'scoot-logo-'));
try {
  const render = (name, sizes, out) => {
    tauriIcon([join(here, name), '-o', out, '-p', sizes.join(',')]);
    return Object.fromEntries(sizes.map((s) => [s, readFileSync(join(out, `${s}x${s}.png`))]));
  };
  const small = render('icon-small.svg', [16, 20, 24, 32], join(tmp, 'small'));
  const large = render('icon.svg', [48, 64, 256], join(tmp, 'large'));
  writeFileSync(join(icons, '32x32.png'), small[32]);
  const entries = { ...small, ...large };
  writeFileSync(
    join(icons, 'icon.ico'),
    ico(Object.entries(entries).map(([size, data]) => ({ size: +size, data }))),
  );

  // Tray icons, one per tray scale (100%, 125%, 150%, 200%)
  mkdirSync(join(icons, 'tray'), { recursive: true });
  for (const theme of ['light', 'dark']) {
    const pngs = render(`tray-${theme}.svg`, [16, 20, 24, 32], join(tmp, theme));
    for (const [s, data] of Object.entries(pngs))
      writeFileSync(join(icons, 'tray', `${theme}-${s}.png`), data);
  }
} finally {
  rmSync(tmp, { recursive: true, force: true });
}
