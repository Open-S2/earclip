const svg = `
<svg xmlns="http://www.w3.org/2000/svg" width="104" height="20">
  <script/>
  <linearGradient id="a" x2="0" y2="100%">
    <stop offset="0" stop-color="#bbb" stop-opacity=".1"/>
    <stop offset="1" stop-opacity=".1"/>
  </linearGradient>
  <rect rx="3" width="60" height="20" fill="#555"/>
  <rect rx="3" x="50" width="54" height="20" fill="@color@"/>
  <path fill="@color@" d="M64 0h4v20h-4z"/>
  <rect rx="0" x="5" width="50" height="20" fill="#555"/>
  <rect rx="3" width="104" height="20" fill="url(#a)"/>
  <g fill="#fff" text-anchor="middle" font-family="DejaVu Sans,Verdana,Geneva,sans-serif" font-size="11">
    <text x="27" y="15" fill="#010101" fill-opacity=".3">code-cov</text>
    <text x="27" y="14">code-cov</text>
    <text x="80" y="15" fill="#010101" fill-opacity=".3">@ratio@</text>
    <text x="80" y="14">@ratio@</text>
  </g>
</svg>
`.trim();

// Run the command and capture stdout
const proc = Bun.spawn(['bun', 'run', 'test:coverage', '2>&1']);
const text = await new Response(proc.stdout).text();
const lines = text.trim().split('\n');

// find line with 'All files'
const line = lines.find((line) => line.includes('All files'));
if (!line) throw new Error('Could not find line with All files');

// line looks like "All files                                |   99.53 |   99.87 |", we want the last number
const percentDocumented = Number(line.split('|')[2].trim().split(' ')[0].replace('%', ''));
if (percentDocumented > 100 || percentDocumented < 0)
  throw new Error('Could not find percent documented');

// build color
const color = percentDocumented < 50 ? '#db654f' : percentDocumented < 90 ? '#dab226' : '#4fc921';

// build badge
const badge = svg.replace(/@ratio@/g, `${percentDocumented}%`).replace(/@color@/g, color);

Bun.write(`${__dirname}/../assets/code-coverage.svg`, badge);

export {};
