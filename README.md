<div align="center" style="background-color:#efeeeb;padding:20px">
  <img src="https://raw.githubusercontent.com/dev-five-git/braillify/main/media/logo.svg" alt="Braillify" width="300" />
</div>

<h3 align="center">
  실시간 한글 점역 라이브러리
</h3>

<div align="center">

---

<img src='https://img.shields.io/npm/v/braillify'>
<img src='https://img.shields.io/bundlephobia/minzip/braillify'>
<img alt="Github Checks" src="https://badgen.net/github/checks/dev-five-git/braillify"/>
<img alt="Apache-2.0 License" src="https://img.shields.io/github/license/dev-five-git/braillify"/>
<a href="https://www.npmjs.com/package/braillify">
<img alt="NPM Downloads" src="https://img.shields.io/npm/dm/braillify.svg?style=flat"/>
</a>
<a href="https://badgen.net/github/stars/dev-five-git/braillify">
<img alt="Github Stars" src="https://badgen.net/github/stars/dev-five-git/braillify" />
</a>
<a href="https://discord.gg/8zjcGc7cWh">
<img alt="Discord" src="https://img.shields.io/discord/1321362173619994644.svg?label=&logo=discord&logoColor=ffffff&color=7389D8&labelColor=6A7EC2" />
</a>
<a href="https://codecov.io/gh/dev-five-git/braillify" >
 <img src="https://codecov.io/gh/dev-five-git/braillify/graph/badge.svg?token=8I5GMB2X5B"/>
</a>
<a href="https://pypi.org/project/braillify" target="_blank">
    <img src="https://img.shields.io/pypi/v/braillify?color=%2334D058&label=pypi%20package" alt="Package version">
</a>
<a href="https://pypi.org/project/braillify" target="_blank">
    <img src="https://img.shields.io/pypi/pyversions/braillify.svg?color=%2334D058" alt="Supported Python versions">
</a>

</div>

---

## 📖 소개

**braillify**는 **Braille(점자)**에 **-ify(~화하다)**를 더해, 한층 더 쉬운 점자화를 보다 널리 퍼뜨리고자 만든 프로젝트입니다.

모두가 점역을 이해하고 활용할 수 있는 환경을 함께 만들어갑니다.

---

## ✨ braillify의 특징

### 01 📋 2024 개정 한국 점자 규정 기반 점역기

braillify는 **2024년 개정된 한국 점자 규정**을 기반으로 설계되고 구현된 점역기입니다.

더 이상 유지보수가 어렵고, 레거시 코드에 의존해 최신 규정과 맞지 않는 점역기를 사용할 필요가 없습니다. 글의 문맥을 고려해 다양한 경우의 수를 판단하여 더욱 자연스럽고 정확한 점역 결과를 제공합니다.

### 02 🌐 완전한 오픈소스 프로젝트

기존에도 점사랑, 하상브레일 등 다양한 점역기가 존재했고, 일부는 API를 제공하기도 했습니다.

하지만 이들은 대부분 소스가 공개되지 않았고, 점역을 위해 API 서버에 연결해야 했습니다. **braillify는 다릅니다.** 누구나 접근하고, 함께 개선해 나갈 수 있도록 점자 표준 구현 전 과정을 오픈소스로 제공합니다.

### 03 🦀 Rust 기반 크로스 플랫폼

braillify는 **Rust 언어**로 개발되었으며, **Node.js**, **Rust**, **Python** 환경을 모두 지원합니다.

또한 **WebAssembly(wasm)** 도 지원하여, 네트워크나 외부 연결 없이 자신의 PC에서 바로 실행 가능한 구조를 가지고 있습니다. 이를 통해 플랫폼에 구애받지 않고 더 자유롭고 유연한 활용이 가능합니다.

원하는 플랫폼이 있다면 Devfive와 함께 braillify를 확장하고 발전시켜보세요.

---

## 📦 설치

### Node.js

[![npm](https://img.shields.io/npm/v/braillify?logo=npm)](https://www.npmjs.com/package/braillify)

```bash
npm install braillify
```

### Python

[![PyPI](https://img.shields.io/pypi/v/braillify?logo=pypi)](https://pypi.org/project/braillify/)

```bash
pip install braillify
```

### Rust

[![Crates.io](https://img.shields.io/crates/v/braillify?logo=rust)](https://crates.io/crates/braillify)

```bash
cargo add braillify
```
