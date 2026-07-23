use braillify as braillify_core;
use braillify::cli::run_cli;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

#[pyfunction]
fn encode(text: &str) -> PyResult<Vec<u8>> {
    braillify_core::encode(text).map_err(PyErr::new::<PyValueError, _>)
}

#[pyfunction]
fn translate_to_unicode(text: &str) -> PyResult<String> {
    braillify_core::encode_to_unicode(text).map_err(PyErr::new::<PyValueError, _>)
}

#[pyfunction]
fn translate_to_braille_font(text: &str) -> PyResult<String> {
    braillify_core::encode_to_braille_font(text).map_err(PyErr::new::<PyValueError, _>)
}

#[pyfunction]
fn cli(py: Python) -> PyResult<()> {
    run_cli(
        py.import("sys")?
            .getattr("argv")?
            .extract::<Vec<String>>()?,
    )
    .map_err(|e| PyValueError::new_err(e.to_string()))
}

/// A Python module implemented in Rust.
#[pymodule(name = "braillify")]
fn lib_braillify(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(encode, m)?)?;
    m.add_function(wrap_pyfunction!(translate_to_unicode, m)?)?;
    m.add_function(wrap_pyfunction!(translate_to_braille_font, m)?)?;
    m.add_function(wrap_pyfunction!(cli, m)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    //! PyO3 binding tests.
    //!
    //! `auto-initialize` (enabled as a dev-dep feature) starts the embedded
    //! Python interpreter before each test, so `Python::with_gil` is usable
    //! without an external Python process.
    use super::*;

    #[test]
    fn encode_happy_path_returns_bytes() {
        Python::attach(|_py| {
            let result = encode("안녕").expect("encode must succeed");
            assert!(!result.is_empty());
        });
    }

    #[test]
    fn encode_engine_failure_maps_to_pyerr() {
        Python::attach(|_py| {
            // 😀 is not a supported CharType → core encode returns Err →
            // mapped to PyValueError via map_err.
            let result = encode("😀");
            assert!(result.is_err());
        });
    }

    #[test]
    fn translate_to_unicode_happy_path() {
        Python::attach(|_py| {
            let result = translate_to_unicode("hi").expect("must succeed");
            assert!(!result.is_empty());
            // Output must be Braille Unicode (U+2800..=U+28FF).
            for ch in result.chars() {
                let cp = ch as u32;
                assert!((0x2800..=0x28FF).contains(&cp), "non-braille char {ch:?}");
            }
        });
    }

    #[test]
    fn translate_to_unicode_failure_maps_to_pyerr() {
        Python::attach(|_py| {
            assert!(translate_to_unicode("😀").is_err());
        });
    }

    #[test]
    fn translate_to_braille_font_happy_path() {
        Python::attach(|_py| {
            let result = translate_to_braille_font("hi").expect("must succeed");
            assert!(!result.is_empty());
        });
    }

    #[test]
    fn translate_to_braille_font_failure_maps_to_pyerr() {
        Python::attach(|_py| {
            assert!(translate_to_braille_font("😀").is_err());
        });
    }

    #[test]
    fn cli_dispatches_with_argv() {
        Python::attach(|py| {
            // Set sys.argv before invoking the CLI shim.
            let sys = py.import("sys").expect("import sys");
            sys.setattr("argv", vec!["braillify".to_string(), "안녕".to_string()])
                .expect("setattr argv");
            let _ = cli(py); // run_one_shot writes to stdout; we just want apply coverage
        });
    }

    #[test]
    fn cli_invalid_input_returns_pyerr() {
        Python::attach(|py| {
            let sys = py.import("sys").expect("import sys");
            sys.setattr("argv", vec!["braillify".to_string(), "😀".to_string()])
                .expect("setattr argv");
            let result = cli(py);
            assert!(result.is_err());
        });
    }

    /// Exercises the `#[pymodule]` registration body — adds every wrapped
    /// pyfunction to a fresh `PyModule` instance. Covers lines 33-38.
    #[test]
    fn pymodule_registers_all_functions() {
        Python::attach(|py| {
            let m = PyModule::new(py, "braillify").expect("module");
            lib_braillify(&m).expect("module init");
            // All four functions must be attached.
            for name in [
                "encode",
                "translate_to_unicode",
                "translate_to_braille_font",
                "cli",
            ] {
                assert!(m.getattr(name).is_ok(), "missing function {name}");
            }
        });
    }
}
