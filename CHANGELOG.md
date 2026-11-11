# Changelog

All notable changes to this project will be documented in this file.

This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.1.0 - 2026-11-11

### 💫 Features

- Implement terminal-aware JSON pretty printing - ([fdd2108](https://github.com/noirbizarre/console-json/commit/fdd2108ed0dd3f870d30a394e1f8e0a9cb244d2c))

### 🐛 Bug Fixes

- Render redirected stdout as compact uncolored JSON - ([ce6cb72](https://github.com/noirbizarre/console-json/commit/ce6cb725376229512e1aaaf28852d1a4015481c2))
- Restrict MSRV validation to library targets - ([b359343](https://github.com/noirbizarre/console-json/commit/b3593436d89e5937b873f19b1418ac4b42b3d6f1))
- Publish the validated release ref consistently - ([5b4d6f5](https://github.com/noirbizarre/console-json/commit/5b4d6f58e698012f73db542dde090777b48c908c))
- Initialize Windows ANSI support for empty NO_COLOR - ([ea79ca3](https://github.com/noirbizarre/console-json/commit/ea79ca3171a6e952b7ea7293008af65925a96576))
-  🚨 **breaking** Remove unused I/O error variant - ([5127ee5](https://github.com/noirbizarre/console-json/commit/5127ee5b21e1455f6c38d376798fe7d77c2dd3ed))
- Leave serde_json map ordering to consumers - ([49f916c](https://github.com/noirbizarre/console-json/commit/49f916cd5da678eda50616c5dbe46b27842a95f8))
- Ignore empty NO_COLOR in automatic color detection - ([ebe724d](https://github.com/noirbizarre/console-json/commit/ebe724d9be20d4fe73acd11e9c53e6850b9f1c4c))

### 📚 Documentation

- Correct placeholder artwork guidance - ([f6f611b](https://github.com/noirbizarre/console-json/commit/f6f611b7c46d21cd784435d4289deef45c83fcdc))
- Explain check-only Rust formatting hooks - ([1cea4f2](https://github.com/noirbizarre/console-json/commit/1cea4f2ed0c5e063fc2a64f2cc21fdb2e9be6b6b))
- Clarify the local CI task scope - ([e97ce06](https://github.com/noirbizarre/console-json/commit/e97ce06dbcf2877dbad4175c01b14107ed2e8dea))
- Align release comments with crate publication - ([cd666f6](https://github.com/noirbizarre/console-json/commit/cd666f64cddd01dcd1c4d4e2b13d3bee446e7e0a))
- Qualify local and CI toolchain parity - ([c882e79](https://github.com/noirbizarre/console-json/commit/c882e794ec8a5b58e39484c094da910c2cf96143))
- Clarify template update worktree effects - ([e86b3ba](https://github.com/noirbizarre/console-json/commit/e86b3ba73a4dd504925f92e2feb15324a3d31c0e))

### 🏗️ Build

- **deps** Update rstest requirement from 0.26 to 0.28 - ([559865f](https://github.com/noirbizarre/console-json/commit/559865fbd9663f71a13da6e60dd7bf3023c9470d))
- **tpl** Bootstrap the project with https://github.com/noirbizarre/rust.tpl - ([689b5f0](https://github.com/noirbizarre/console-json/commit/689b5f0a553b26c5fed8956b2ba13ba0f70d9fc0))

### Tpl

- Render rust at main - ([1006bd2](https://github.com/noirbizarre/console-json/commit/1006bd2495e2b648d9eb2cf1a2af9656601b9ae4))

## ❤️ New Contributors

* @noirbizarre made their first contribution
