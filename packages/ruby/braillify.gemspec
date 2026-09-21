# frozen_string_literal: true

require_relative "lib/braillify/version"

Gem::Specification.new do |spec|
  spec.name = "braillify"
  spec.version = Braillify::VERSION
  spec.authors = ["JeongMin Oh"]
  spec.email = ["owjs39@gmail.com"]

  spec.summary = "Rust 기반 크로스플랫폼 한국어 점역 라이브러리"
  spec.description = "한국어 텍스트를 한국 점자(2024 개정 한국 점자 규정)로 변환하는 라이브러리. Rust 네이티브 확장으로 동작한다."
  spec.homepage = "https://braillify.kr"
  spec.license = "Apache-2.0"
  spec.required_ruby_version = ">= 3.1"

  spec.metadata["homepage_uri"] = spec.homepage
  spec.metadata["source_code_uri"] = "https://github.com/dev-five-git/braillify"

  spec.files = Dir[
    "lib/**/*.rb",
    "src/**/*.rs",
    "Cargo.toml",
    "build.rs",
    "extconf.rb",
    "README.md"
  ]
  spec.require_paths = ["lib"]
  spec.extensions = ["extconf.rb"]

  # 소스 gem 설치 시 extconf.rb가 rb_sys/mkmf를 require한다.
  spec.add_dependency "rb_sys", "~> 0.9"
end
