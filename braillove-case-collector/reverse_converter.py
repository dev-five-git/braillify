"""
Unicode 점자 → internal / expected 역변환기

Usage:
    python reverse_converter.py
    (유니코드 점자 문자열 입력 시 internal과 expected 출력)
"""

# 기본 64셀 패턴 (인덱스 0-63)
pattern = " a1b'k2l@cif/msp\"e3h9o6r^djg>ntq,*5<-u8v.%[$+x!&;:4\\0z7(_?w]#y)="
braille = "⠀⠁⠂⠃⠄⠅⠆⠇⠈⠉⠊⠋⠌⠍⠎⠏⠐⠑⠒⠓⠔⠕⠖⠗⠘⠙⠚⠛⠜⠝⠞⠟⠠⠡⠢⠣⠤⠥⠦⠧⠨⠩⠪⠫⠬⠭⠮⠯⠰⠱⠲⠳⠴⠵⠶⠷⠸⠹⠺⠻⠼⠽⠾⠿"

# 특수 매핑 (pattern에 없는 문자들)
# internal char → braille index
SPECIAL_FORWARD = {
    "`": 0,   # 백틱 → 인덱스 0 (공백과 동일)
    "{": 42,  # 여는 중괄호
    "}": 59,  # 닫는 중괄호
    "~": 24,  # 물결표
    "|": 51,  # 파이프
}

# braille index → internal char (역매핑은 pattern 우선, 특수문자는 별도 처리 필요)
# 특수 매핑의 역방향: 같은 인덱스에 여러 internal 문자가 매핑될 수 있음
# - 인덱스 0: 공백(' ') 또는 백틱('`')
# - 인덱스 42: '*' (pattern) 또는 '{' (special)
# - 인덱스 59: ']' (pattern) 또는 '}' (special)
# - 인덱스 24: '^' (pattern) 또는 '~' (special)
# - 인덱스 51: '%' (pattern) 또는 '|' (special)
# 
# 기본적으로 pattern의 문자를 우선 사용 (원본 converter.py가 internal→unicode 변환 시
# pattern에 있으면 pattern 사용, 없으면 special 사용하므로)


def normalize_unicode(unicode_str: str) -> str:
    """
    일반 스페이스(U+0020)를 점자 스페이스(U+2800)로 변환
    """
    return unicode_str.replace(' ', '⠀')  # U+0020 → U+2800


def unicode_to_internal(unicode_str: str) -> str:
    """
    유니코드 점자 문자열을 internal 표기로 변환
    
    Args:
        unicode_str: 점자 유니코드 문자열 (예: "⠼⠙⠌⠉")
    
    Returns:
        internal 표기 문자열 (예: "#d/c")
    """
    unicode_str = normalize_unicode(unicode_str)
    result = []
    for char in unicode_str:
        if char in braille:
            idx = braille.index(char)
            result.append(pattern[idx])
        else:
            raise ValueError(f"Unknown braille character: {char} (U+{ord(char):04X})")
    return "".join(result)


def unicode_to_expected(unicode_str: str) -> str:
    """
    유니코드 점자 문자열을 expected (인덱스 문자열)로 변환
    
    Args:
        unicode_str: 점자 유니코드 문자열 (예: "⠼⠙⠌⠉")
    
    Returns:
        expected 인덱스 문자열 (예: "6025129")
    """
    unicode_str = normalize_unicode(unicode_str)
    result = []
    for char in unicode_str:
        if char in braille:
            idx = braille.index(char)
            result.append(str(idx))
        else:
            raise ValueError(f"Unknown braille character: {char} (U+{ord(char):04X})")
    return "".join(result)


def convert_unicode(unicode_str: str) -> tuple[str, str]:
    """
    유니코드 점자를 internal과 expected로 동시에 변환
    
    Args:
        unicode_str: 점자 유니코드 문자열
    
    Returns:
        (internal, expected) 튜플
    """
    internal = unicode_to_internal(unicode_str)
    expected = unicode_to_expected(unicode_str)
    return internal, expected


def main():
    """대화형 모드: 유니코드 점자 입력 → internal, expected 출력"""
    print("Unicode 점자 → internal / expected 역변환기")
    print("점자 유니코드 문자열을 입력하세요 (빈 줄 입력 시 종료)")
    print("-" * 50)
    
    while True:
        try:
            inp = input("unicode> ").strip()
            if not inp:
                break
            
            internal, expected = convert_unicode(inp)
            print(f"internal: {internal}")
            print(f"expected: {expected}")
            print()
        except ValueError as e:
            print(f"Error: {e}")
            print()
        except EOFError:
            break


if __name__ == "__main__":
    main()
