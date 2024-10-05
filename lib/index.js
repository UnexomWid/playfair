/* tslint:disable */
/* eslint-disable */
/* prettier-ignore */

import { existsSync, readFileSync } from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const { platform, arch } = process

let nativeBinding = null
let localFileExisted = false
let loadError = null

// Since ESM doesn't support loading .node files,
// we have to use require()
import { createRequire } from "module";
const require = createRequire(import.meta.url)

function isMusl() {
  // For Node 10
  if (!process.report || typeof process.report.getReport !== 'function') {
    try {
      const lddPath = require('child_process').execSync('which ldd').toString().trim()
      return readFileSync(lddPath, 'utf8').includes('musl')
    } catch (e) {
      return true
    }
  } else {
    const { glibcVersionRuntime } = process.report.getReport().header
    return !glibcVersionRuntime
  }
}

const cwd = process.cwd();
process.chdir(path.join(__dirname, '..'));

switch (platform) {
  case 'android':
    switch (arch) {
      case 'arm64':
        localFileExisted = existsSync(path.join(__dirname, 'package.android-arm64.node'))
        try {
          if (localFileExisted) {
            nativeBinding = require('./package.android-arm64.node')
          } else {
            nativeBinding = require('package-android-arm64')
          }
        } catch (e) {
          loadError = e
        }
        break
      case 'arm':
        localFileExisted = existsSync(path.join(__dirname, 'package.android-arm-eabi.node'))
        try {
          if (localFileExisted) {
            nativeBinding = require('./package.android-arm-eabi.node')
          } else {
            nativeBinding = require('package-android-arm-eabi')
          }
        } catch (e) {
          loadError = e
        }
        break
      default:
        throw new Error(`Unsupported architecture on Android ${arch}`)
    }
    break
  case 'win32':
    switch (arch) {
      case 'x64':
        localFileExisted = existsSync(
          path.join(__dirname, 'package.win32-x64-msvc.node')
        )
        try {
          if (localFileExisted) {
            nativeBinding = require('./package.win32-x64-msvc.node')
          } else {
            nativeBinding = require('package-win32-x64-msvc')
          }
        } catch (e) {
          loadError = e
        }
        break
      case 'ia32':
        localFileExisted = existsSync(
          path.join(__dirname, 'package.win32-ia32-msvc.node')
        )
        try {
          if (localFileExisted) {
            nativeBinding = require('./package.win32-ia32-msvc.node')
          } else {
            nativeBinding = require('package-win32-ia32-msvc')
          }
        } catch (e) {
          loadError = e
        }
        break
      case 'arm64':
        localFileExisted = existsSync(
          path.join(__dirname, 'package.win32-arm64-msvc.node')
        )
        try {
          if (localFileExisted) {
            nativeBinding = require('./package.win32-arm64-msvc.node')
          } else {
            nativeBinding = require('package-win32-arm64-msvc')
          }
        } catch (e) {
          loadError = e
        }
        break
      default:
        throw new Error(`Unsupported architecture on Windows: ${arch}`)
    }
    break
  case 'darwin':
    localFileExisted = existsSync(path.join(__dirname, 'package.darwin-universal.node'))
    try {
      if (localFileExisted) {
        nativeBinding = require('./package.darwin-universal.node')
      } else {
        nativeBinding = require('package-darwin-universal')
      }
      break
    } catch {}
    switch (arch) {
      case 'x64':
        localFileExisted = existsSync(path.join(__dirname, 'package.darwin-x64.node'))
        try {
          if (localFileExisted) {
            nativeBinding = require('./package.darwin-x64.node')
          } else {
            nativeBinding = require('package-darwin-x64')
          }
        } catch (e) {
          loadError = e
        }
        break
      case 'arm64':
        localFileExisted = existsSync(
          path.join(__dirname, 'package.darwin-arm64.node')
        )
        try {
          if (localFileExisted) {
            nativeBinding = require('./package.darwin-arm64.node')
          } else {
            nativeBinding = require('package-darwin-arm64')
          }
        } catch (e) {
          loadError = e
        }
        break
      default:
        throw new Error(`Unsupported architecture on macOS: ${arch}`)
    }
    break
  case 'freebsd':
    if (arch !== 'x64') {
      throw new Error(`Unsupported architecture on FreeBSD: ${arch}`)
    }
    localFileExisted = existsSync(path.join(__dirname, 'package.freebsd-x64.node'))
    try {
      if (localFileExisted) {
        nativeBinding = require('./package.freebsd-x64.node')
      } else {
        nativeBinding = require('package-freebsd-x64')
      }
    } catch (e) {
      loadError = e
    }
    break
  case 'linux':
    switch (arch) {
      case 'x64':
        if (isMusl()) {
          localFileExisted = existsSync(
            path.join(__dirname, 'package.linux-x64-musl.node')
          )
          try {
            if (localFileExisted) {
              nativeBinding = require('./package.linux-x64-musl.node')
            } else {
              nativeBinding = require('package-linux-x64-musl')
            }
          } catch (e) {
            loadError = e
          }
        } else {
          localFileExisted = existsSync(
            path.join(__dirname, 'package.linux-x64-gnu.node')
          )
          try {
            if (localFileExisted) {
              nativeBinding = require('./package.linux-x64-gnu.node')
            } else {
              nativeBinding = require('package-linux-x64-gnu')
            }
          } catch (e) {
            loadError = e
          }
        }
        break
      case 'arm64':
        if (isMusl()) {
          localFileExisted = existsSync(
            path.join(__dirname, 'package.linux-arm64-musl.node')
          )
          try {
            if (localFileExisted) {
              nativeBinding = require('./package.linux-arm64-musl.node')
            } else {
              nativeBinding = require('package-linux-arm64-musl')
            }
          } catch (e) {
            loadError = e
          }
        } else {
          localFileExisted = existsSync(
            path.join(__dirname, 'package.linux-arm64-gnu.node')
          )
          try {
            if (localFileExisted) {
              nativeBinding = require('./package.linux-arm64-gnu.node')
            } else {
              nativeBinding = require('package-linux-arm64-gnu')
            }
          } catch (e) {
            loadError = e
          }
        }
        break
      case 'arm':
        if (isMusl()) {
          localFileExisted = existsSync(
            path.join(__dirname, 'package.linux-arm-musleabihf.node')
          )
          try {
            if (localFileExisted) {
              nativeBinding = require('./package.linux-arm-musleabihf.node')
            } else {
              nativeBinding = require('package-linux-arm-musleabihf')
            }
          } catch (e) {
            loadError = e
          }
        } else {
          localFileExisted = existsSync(
            path.join(__dirname, 'package.linux-arm-gnueabihf.node')
          )
          try {
            if (localFileExisted) {
              nativeBinding = require('./package.linux-arm-gnueabihf.node')
            } else {
              nativeBinding = require('package-linux-arm-gnueabihf')
            }
          } catch (e) {
            loadError = e
          }
        }
        break
      case 'riscv64':
        if (isMusl()) {
          localFileExisted = existsSync(
            path.join(__dirname, 'package.linux-riscv64-musl.node')
          )
          try {
            if (localFileExisted) {
              nativeBinding = require('./package.linux-riscv64-musl.node')
            } else {
              nativeBinding = require('package-linux-riscv64-musl')
            }
          } catch (e) {
            loadError = e
          }
        } else {
          localFileExisted = existsSync(
            path.join(__dirname, 'package.linux-riscv64-gnu.node')
          )
          try {
            if (localFileExisted) {
              nativeBinding = require('./package.linux-riscv64-gnu.node')
            } else {
              nativeBinding = require('package-linux-riscv64-gnu')
            }
          } catch (e) {
            loadError = e
          }
        }
        break
      case 's390x':
        localFileExisted = existsSync(
          path.join(__dirname, 'package.linux-s390x-gnu.node')
        )
        try {
          if (localFileExisted) {
            nativeBinding = require('./package.linux-s390x-gnu.node')
          } else {
            nativeBinding = require('package-linux-s390x-gnu')
          }
        } catch (e) {
          loadError = e
        }
        break
      default:
        throw new Error(`Unsupported architecture on Linux: ${arch}`)
    }
    break
  default:
    throw new Error(`Unsupported OS: ${platform}, architecture: ${arch}`)
}

process.chdir(cwd);

if (!nativeBinding) {
  if (loadError) {
    throw loadError
  }
  throw new Error(`Failed to load native binding`)
}

const { binding } = nativeBinding;

export default await import(binding);
