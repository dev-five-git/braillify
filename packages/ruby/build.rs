fn main() -> Result<(), Box<dyn std::error::Error>> {
    // rb-sys-test-helpers 기반 `cargo test`가 임베디드 Ruby VM에 링크할 수 있도록
    // rbconfig에서 링크 플래그/cfg를 활성화한다.
    let _ = rb_sys_env::activate()?;
    Ok(())
}
