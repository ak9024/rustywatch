---
title: Benchmarking
sidebar:
  order: 1
  badge:
    text: New
    variant: tip
---

I conducted a performance comparison between `rustywatch v0.2.10` and `air v1.52.3`, using a 1-second timeout to exit the shell after running each CLI.

For benchmarking, I used the https://github.com/sharkdp/hyperfine

The tests were performed on my MacBook Air M2.

Results:
RustyWatch is 1.00x faster than Go Air.

More: https://www.linkedin.com/feed/update/urn:li:activity:7248331520506003456/

