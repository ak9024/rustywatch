---
title: Simple setup
description: Simple setup page
---

```shell
# create go project
mkdir go-project;
cd go-project;
# init go project
go mod init go-project;
touch main.go;
```

```
.
├── go.mod
├── main.go
└── rustywatch.yaml
```

Create a simple go project with just print `Hello World RustyWatch`.

```go
package main

import (
	"fmt"
)

func main() {
	fmt.Println("Hello World RustyWatch")
}
```

Then please create `rustywatch.yaml` in your project.

```yaml
workspaces:
  # first project binary apps
  - dir: '.'
    cmd: # define command to build binary
    - go build main.go
    bin_path: './main' # define path for binary location
    ignore:
     - '.git'
```

Open your shell then type `rustywatch`

```shell

╭━━━╮╱╱╱╱╱╭╮╱╱╱╱╭╮╭╮╭╮╱╱╭╮╱╱╱╭╮
┃╭━╮┃╱╱╱╱╭╯╰╮╱╱╱┃┃┃┃┃┃╱╭╯╰╮╱╱┃┃
┃╰━╯┣╮╭┳━┻╮╭╋╮╱╭┫┃┃┃┃┣━┻╮╭╋━━┫╰━╮
┃╭╮╭┫┃┃┃━━┫┃┃┃╱┃┃╰╯╰╯┃╭╮┃┃┃╭━┫╭╮┃
┃┃┃╰┫╰╯┣━━┃╰┫╰━╯┣╮╭╮╭┫╭╮┃╰┫╰━┫┃┃┃
╰╯╰━┻━━┻━━┻━┻━╮╭╯╰╯╰╯╰╯╰┻━┻━━┻╯╰╯
╱╱╱╱╱╱╱╱╱╱╱╱╭━╯┃
╱╱╱╱╱╱╱╱╱╱╱╱╰━━╯
[WARN ] Remove old binary: ./main
[WARN ] Old binary removed
[INFO ] Restarting binary: ./main
[INFO ] Binary started: 34009
[INFO ] Waching directory: "."
Hello World RustyWatch
```

RustyWatch run on your root project, then listen all your changes in root based on your configuration.

```yaml
workspaces:
- dir: <your_dir_to_watch_by_rustywatch>
```

