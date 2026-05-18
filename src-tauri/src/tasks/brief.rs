//! Stub for Task 6 compilation. Task 8 replaces this whole file with the
//! real renderer + writer.

use crate::{error::AppError, models::Task};

pub fn write_brief(_task: &Task) -> Result<(), AppError> {
    Ok(())
}
