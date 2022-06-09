# 오소리 Rust 엔진

## Introduction
오소리의 엔진을 C에서 Rust로 전환하기 위한 프로젝트입니다.

## build

```bash
$ cargo build
```

개발 중에 반복적으로 컴파일에 실패한다면, `build`가 아닌 `check` 명령을 사용하는 것이 효율적입니다.

```bash
$ cargo check
```

## boot

실행시 admin의 ip 주소 등이 필요합니다.
```bash
$ cargo run -- -a 127.0.0.1:3030
```

## format

formatter는 rustfmt를 사용합니다.
```bash
$ cargo fmt
```

커밋 전에 rustfmt를 실행해주세요.

[저장할때마다 fmt 실행하기(intellij)](https://plugins.jetbrains.com/plugin/8182-rust/docs/rust-code-style-and-formatting.html)

[vscode-rustfmt](https://marketplace.visualstudio.com/items?itemName=statiolake.vscode-rustfmt)