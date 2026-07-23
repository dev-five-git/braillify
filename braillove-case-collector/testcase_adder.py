"""
테스트 케이스 자동 추가 도구

Usage:
    python testcase_adder.py

1. JSON 파일 경로 입력 (또는 번호로 선택)
2. "input,unicode" 형식으로 입력
3. 자동으로 internal/expected 계산 후 JSON에 추가
"""

import json
import os
import sys
from pathlib import Path

# reverse_converter에서 함수 가져오기
from reverse_converter import convert_unicode, normalize_unicode

# 프로젝트 루트 기준 test_cases 경로
SCRIPT_DIR = Path(__file__).resolve().parent
PROJECT_ROOT = SCRIPT_DIR.parent
TEST_CASES_DIR = PROJECT_ROOT / "test_cases"


def list_json_files() -> list[Path]:
    """test_cases 폴더 내 모든 JSON 파일 목록 반환"""
    json_files = []
    for root, dirs, files in os.walk(TEST_CASES_DIR):
        for file in files:
            if file.endswith('.json'):
                json_files.append(Path(root) / file)
    return sorted(json_files)


def select_json_file() -> Path | None:
    """JSON 파일 선택 (번호 또는 경로)"""
    json_files = list_json_files()

    print("\n=== 사용 가능한 JSON 파일 ===")
    for i, f in enumerate(json_files, 1):
        rel_path = f.relative_to(PROJECT_ROOT)
        print(f"  {i}. {rel_path}")
    print()

    while True:
        try:
            choice = input("파일 번호 또는 경로 입력 (q=종료): ").strip()
        except KeyboardInterrupt:
            # Ctrl+C가 눌리면 바로 죽지 않고 정말 종료할지 다시 묻는다
            if confirm_exit(0):
                return None
            print("\n계속합니다.\n")
            continue

        if choice.lower() == 'q':
            return None

        # 번호로 선택
        if choice.isdigit():
            idx = int(choice) - 1
            if 0 <= idx < len(json_files):
                return json_files[idx]
            print(f"잘못된 번호입니다. 1-{len(json_files)} 사이로 입력하세요.")
            continue

        # 경로로 선택
        path = Path(choice)
        if not path.is_absolute():
            path = PROJECT_ROOT / choice

        if path.exists() and path.suffix == '.json':
            return path

        print("파일을 찾을 수 없습니다. 다시 입력하세요.")


def load_json(path: Path) -> list:
    """JSON 파일 로드"""
    with open(path, 'r', encoding='utf-8') as f:
        return json.load(f)


def save_json(path: Path, data: list):
    """JSON 파일 저장 (2칸 들여쓰기)"""
    resolved_path = path.resolve()
    with open(resolved_path, 'w', encoding='utf-8') as f:
        json.dump(data, f, ensure_ascii=False, indent=2)
    # 저장 후 검증
    with open(resolved_path, 'r', encoding='utf-8') as f:
        saved_data = json.load(f)
    if len(saved_data) != len(data):
        raise RuntimeError(f"저장 검증 실패: 예상 {len(data)}개, 실제 {len(saved_data)}개")


def confirm_exit(added_count: int) -> bool:
    """Ctrl+C 시 정말 종료할지 확인. 종료하면 True, 계속하면 False."""
    try:
        msg = "\n정말 종료하시겠습니까? (y=종료 / 그 외=계속"
        if added_count > 0:
            msg += f", 미저장 {added_count}개"
        msg += "): "
        answer = input(msg).strip().lower()
    except (KeyboardInterrupt, EOFError):
        # 확인 도중 다시 Ctrl+C(또는 EOF)면 즉시 종료
        print()
        return True
    return answer == 'y'


def create_entry(input_text: str, unicode_text: str) -> dict:
    """입력값으로 테스트 케이스 엔트리 생성"""
    # 일반 스페이스를 점자 스페이스로 변환
    unicode_text = normalize_unicode(unicode_text)
    internal, expected = convert_unicode(unicode_text)
    return {
        "input": input_text,
        "internal": internal,
        "expected": expected,
        "unicode": unicode_text
    }


def main():
    print("=" * 50)
    print("   테스트 케이스 자동 추가 도구")
    print("=" * 50)

    # 1. JSON 파일 선택
    json_path = select_json_file()
    if json_path is None:
        print("종료합니다.")
        return

    resolved_path = json_path.resolve()
    print(f"\n선택된 파일: {json_path.relative_to(PROJECT_ROOT)}")
    print(f"절대 경로: {resolved_path}")

    # 2. 기존 데이터 로드
    try:
        data = load_json(json_path)
        print(f"기존 엔트리 수: {len(data)}")
    except FileNotFoundError:
        data = []
        print("새 파일을 생성합니다.")
    except json.JSONDecodeError as e:
        print(f"JSON 파싱 오류: {e}")
        return

    print()
    print("-" * 50)
    print("입력: 첫 줄에 영단어/문장, 두 번째 줄에 점자")
    print("명령: q=종료, l=현재목록, u=마지막삭제, s=저장, f=파일재선택")
    print("-" * 50)
    print()

    added_count = 0

    while True:
        try:
            try:
                user_input = input("추가> ").strip()
            except EOFError:
                break

            if not user_input:
                continue

            # 명령어 처리
            if user_input.lower() == 'q':
                break

            if user_input.lower() == 'l':
                print(f"\n현재 엔트리 수: {len(data)}")
                for i, entry in enumerate(data[-5:], max(1, len(data) - 4)):
                    print(f"  {i}. {entry.get('input', 'N/A')} -> {entry.get('unicode', 'N/A')}")
                print()
                continue

            if user_input.lower() == 'u':
                if data:
                    removed = data.pop()
                    added_count = max(0, added_count - 1)
                    print(f"삭제됨: {removed.get('input', 'N/A')}")
                else:
                    print("삭제할 엔트리가 없습니다.")
                continue

            if user_input.lower() == 's':
                print(f"저장 중... {json_path.resolve()}")
                save_json(json_path, data)
                print(f"저장 완료! (총 {len(data)}개)")
                continue

            if user_input.lower() == 'f':
                # 현재 파일 저장 여부 확인
                if added_count > 0:
                    save_confirm = input(f"{added_count}개 미저장. 저장할까요? (y/n): ").strip().lower()
                    if save_confirm == 'y':
                        save_json(json_path, data)
                        print(f"저장 완료! (총 {len(data)}개)")

                # 새 파일 선택
                new_path = select_json_file()
                if new_path is None:
                    print("파일 선택 취소. 기존 파일 유지.")
                    continue

                json_path = new_path
                resolved_path = json_path.resolve()
                print(f"\n선택된 파일: {json_path.relative_to(PROJECT_ROOT)}")
                print(f"절대 경로: {resolved_path}")

                try:
                    data = load_json(json_path)
                    print(f"기존 엔트리 수: {len(data)}")
                except FileNotFoundError:
                    data = []
                    print("새 파일을 생성합니다.")
                except json.JSONDecodeError as e:
                    print(f"JSON 파싱 오류: {e}")
                    continue

                added_count = 0
                print()
                continue

            # 첫 번째 입력은 영단어/문장 (input)
            input_text = user_input

            # 두 번째 입력: 점자 (unicode)
            try:
                unicode_text = input("점자> ").strip()
            except EOFError:
                break

            if not unicode_text:
                print("취소됨")
                continue

            # 유니코드 점자 검증 (일반 스페이스는 허용 - 점자 스페이스로 변환됨)
            if not all('⠀' <= c <= '⣿' or c == ' ' for c in unicode_text):
                print(f"경고: '{unicode_text}'에 점자가 아닌 문자가 포함되어 있습니다.")
                confirm = input("계속하시겠습니까? (y/n): ").strip().lower()
                if confirm != 'y':
                    continue

            try:
                entry = create_entry(input_text, unicode_text)
                data.append(entry)
                added_count += 1

                print(f"  추가됨: {entry}")
                print()
            except ValueError as e:
                print(f"오류: {e}")
                continue
        except KeyboardInterrupt:
            # Ctrl+C가 눌리면 바로 죽지 않고 정말 종료할지 다시 묻는다
            if confirm_exit(added_count):
                break
            print("\n계속합니다.\n")
            continue

    print("종료합니다.")


if __name__ == "__main__":
    main()
