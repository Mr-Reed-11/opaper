import { readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const identity = JSON.parse(readFileSync(path.join(root, 'identity.json'), 'utf8'));

const tauriConfig = {
  $schema: 'https://schema.tauri.app/config/2',
  productName: identity.name,
  version: '0.1.0',
  identifier: identity.identifier,
  build: {
    beforeDevCommand: 'npm run dev',
    devUrl: 'http://localhost:1420',
    beforeBuildCommand: 'npm run build',
    frontendDist: '../dist',
  },
  app: {
    windows: [
      {
        label: 'main',
        title: identity.name,
        width: 960,
        height: 640,
        resizable: true,
      },
    ],
    security: { csp: null },
  },
  bundle: {
    active: true,
    icon: [
      'icons/32x32.png',
      'icons/128x128.png',
      'icons/128x128@2x.png',
      'icons/icon.ico',
    ],
  },
};

const desktop = `[Desktop Entry]
Type=Application
Version=1.0
Name=${identity.name}
Comment=${identity.comment}
Exec=${identity.name}
Icon=${identity.name}
Terminal=false
Categories=Graphics;2DGraphics;
StartupWMClass=${identity.name}
`;

writeFileSync(
  path.join(root, 'app/src-tauri/tauri.conf.json'),
  `${JSON.stringify(tauriConfig, null, 2)}\n`,
);
writeFileSync(path.join(root, `packaging/${identity.identifier}.desktop`), desktop);
console.log(`generated Tauri config and ${identity.identifier}.desktop`);
