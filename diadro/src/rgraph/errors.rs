use thiserror::Error;

#[derive(Error, Debug)]
pub enum MxErrors {
    #[error("Wrong cell type")]
    WrongMxCellType,
    #[error("Cell not found")]
    MxCellNotFound,
}
