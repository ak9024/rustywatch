---
title: Guides
description: Guides page.
---

## Install

Please install cargo first `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh` if you don't have.

```shell
cargo install rustywatch
```

```shell
# create nodejs project
mkdir nodejs-project
cd nodejs-project
touch index.js
```

Create nodejs project with a simple print `Hello World`

```js
console.log('Hello World')
```

Run `rustywatch` in a single command without `rustywatch.yaml`.

```shell
rustywatch -d '.' -c 'node index.js'
```
