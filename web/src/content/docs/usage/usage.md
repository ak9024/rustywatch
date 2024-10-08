---
title: Usage
---

### Running RustyWatch without configuration.

You can run RustyWatch directly from the terminal without needing a configuration file.

Here's how:


**Basic Usage**

```shell
rustywatch --dir <your_dir> \
  --cmd <your_command_to_build_the_apps> \
  --bin-path <your_bin_location> \
  --ignore '.git/'
```

### Example: Simple HTTP Server with Go 

Let's walk through an example where you create a simple HTTP server using Go and the Fiber web framework.


**Setup the project**

Create new directory `go-project`.

```shell
mkdir go-project;
cd go-project;
```

Initialize go module for your project.

```shell
go mod init go-project;
```

Install the Fiber framework by running:

```shell
go get github.com/gofiber/fiber/v2
```

Now, create a `main.go` file with the following content.

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

Your project structure look like this.

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
