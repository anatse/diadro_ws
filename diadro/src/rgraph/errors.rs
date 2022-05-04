use thiserror::Error;

#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum MxErrors {
    #[error("Wrong cell type")]
    WrongMxCellType,
    #[error("Cell not found")]
    MxCellNotFound,
}
