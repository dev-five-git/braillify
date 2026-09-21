# frozen_string_literal: true

require "mkmf"
require "rb_sys/mkmf"

# Cargo.toml은 이 파일과 같은 디렉토리에 있다.
# 산출물은 lib/braillify/braillify_rb.{so,bundle,dll}로 설치된다.
create_rust_makefile("braillify/braillify_rb")
