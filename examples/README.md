# examples

Check this in the root `../rustywatch.yaml`.

```yaml
workspaces:
  - dir: 'web'
    cmd: 'cd web;npm run dev'
  - dir: 'examples/go-fiber'
    cmd:
      - echo "go-fiber"
      - |
        cd examples/go-fiber;
        go build;
    bin_path: 'examples/go-fiber/go-fiber'
  - dir: 'examples/nodejs'
    cmd:
      - echo "nodejs"
      - cd examples/nodejs;npm start;
  - dir: 'examples/rust-actix'
    cmd:
      - echo "rust-actix"
      - |
        cd examples/rust-actix;
        cargo build;
    bin_path: 'examples/rust-actix/target/debug/rust-actix'
    ignore:
      - 'examples/rust-actix/target/'
  - dir: 'examples/bun'
    cmd:
      - echo "bun expressjs"
      - |
          cd examples/bun;
          bun --watch ./server.ts;
```