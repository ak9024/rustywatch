---
title: Usage
---

**RustyWatch** can be usage without the configuration.

```shell
rustywatch --dir <your_dir> \
  --cmd <your_command_to_build_the_apps> \
  --bin-path <your_bin_location> \
  --ignore '.git/'
```

### Examples

Example you create a simple http server with go.

```shell
mkdir go-project;
cd go-project;
```

Then create or init module with go.

```shell
go mod init go-project;
```

Install web framework, example `go fiber`.

```shell
go get github.com/gofiber/fiber/v2
```

Create `main.go`.

```go
package main

import "github.com/gofiber/fiber/v2"

func main() {
	app := fiber.New()

	app.Get("/", func(c *fiber.Ctx) error {
		return c.SendString("Hello, World!")
	})

	app.Listen(":3000")
}
```
```
.
└── go-project/
    ├── go.mod
    ├── go.sum
    └── main.go

```

Then start a project with RustyWatch.

```shell
cd go-project;
rustywatch -d '.'  -c 'go build' --bin-path 'go-project'
```

In `--bin-path` please suitable with your binary name, or path location where your binary exists.
