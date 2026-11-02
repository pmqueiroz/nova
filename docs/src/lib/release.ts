import { readFileSync } from 'node:fs';
import { join } from 'node:path';

export const version = readFileSync(join(process.cwd(), '..', '.release-please-manifest.json'), 'utf8').match(/"\.":\s*"([^"]+)"/)?.[1];
