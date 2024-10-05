import { exec } from 'node:child_process';
import { mkdir, stat, copyFile } from 'node:fs/promises';
import { join } from 'node:path';

const TARGETS = [{
    'triple': 'x86_64-pc-windows-msvc',
    'ext': '.dll',
    'name': 'win32-x64-msvc'
}, {
    'triple': 'x86_64-unknown-linux-gnu',
    'ext': '.so',
    'name': 'linux-x64-gnu'
}, {
    'triple': 'x86_64-apple-darwin',
    'ext': '.dylib',
    'name': 'darwin-x64'
}];

const DIST_DIR = '../_dist';

async function build(target) {
    return new Promise((resolve, reject) => {
        exec(`cargo build --release --target ${target}`, (err, stdout, stderr) => {

        if (err) {
            return reject(err);
        }
        resolve();
        });
    });
}

function info(what) {
    console.log(`[INFO] ${what}`);
}

function warn(what) {
    console.error(`[WARNING] ${what}`);
}

function err(what) {
    console.error(`[ERROR] ${what}`);
}

try {
    await stat(DIST_DIR);
} catch {
    await mkdir(DIST_DIR);
}

let succeeded = 0;

for (const target of TARGETS) {
    try {
        info(`Building for ${target.triple}`)
        await build(target.triple);

        const src = `../target/${target.triple}/release/playfair${target.ext}`;
        const dest = `${DIST_DIR}/package.${target.name}.node`
        info(`Copying binary to ${dest}`)

        await copyFile(src, dest);

        info('OK');

        succeeded++;
    } catch (ex) {
        err('Build failed!');
        console.error(ex);
    }
}

if (succeeded == 0) {
    err("Build failed for all targets");
} else {
    await copyFile('./index.js', join(DIST_DIR, 'package.mjs'));

    if (succeeded < TARGETS.length) {
        warn('Build failed for some targets');
        warn("This means that Playfair won't work on some platforms")
        warn("You can safely ignore this warning if you only wanted to build for your own OS")
    } else {
        info('Build OK');
    }
}