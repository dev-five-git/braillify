# frozen_string_literal: true

require_relative "braillify/version"

# 프리컴파일 gem은 Ruby 마이너 버전별 디렉토리(lib/braillify/3.4/...)에,
# 소스 빌드는 lib/braillify/ 바로 아래에 확장이 놓인다.
begin
  ruby_minor = RUBY_VERSION[/\d+\.\d+/]
  require_relative "braillify/#{ruby_minor}/braillify_rb"
rescue LoadError
  require_relative "braillify/braillify_rb"
end

# 한국어 텍스트를 한국 점자로 변환하는 모듈 (2024 개정 한국 점자 규정 기반).
# 메서드는 네이티브 확장(Rust)에서 정의된다:
#
#   Braillify.encode(text)                    # => ASCII-8BIT String (점자 셀 바이트)
#   Braillify.translate_to_unicode(text)      # => 점자 유니코드 String
#   Braillify.translate_to_braille_font(text) # => 점자 폰트 String
#
# 변환할 수 없는 입력은 ArgumentError를 발생시킨다.
module Braillify
end
