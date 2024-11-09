mod board;
mod game;
mod result;
mod single_match;
mod multi_match;

pub use result::GameResult;

use crate::py_helper::add_submodule;
use single_match::Match;
use multi_match::MultiMatch;

use pyo3::prelude::{PyModule, PyResult, Python};

pub(crate) fn register_module(py: Python<'_>, prefix: &str, super_mod: &PyModule) -> PyResult<()> {
    let m = PyModule::new(py, "arena")?;
    m.add_class::<Match>()?;
    m.add_class::<MultiMatch>()?;
    add_submodule(py, prefix, super_mod, m)
}
