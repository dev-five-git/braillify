# frozen_string_literal: true

module Braillify
  # 버전 단일 소스는 Cargo.toml — changepacks가 bump하는 파일이다.
  VERSION = File.read(File.expand_path("../../Cargo.toml", __dir__), encoding: "UTF-8")[/^version = "([^"]+)"/, 1].freeze
end
