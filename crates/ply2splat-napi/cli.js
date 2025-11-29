#!/usr/bin/env node

import { cli } from './index.js';

// Get arguments starting from the 3rd one (skip 'node' and 'script_path')
const args = process.argv.slice(2);

try {
  cli(args);
} catch (error) {
  console.error(error.message);
  process.exit(1);
}
