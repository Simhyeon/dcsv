/// VCont is a generic trait for various virtual csv structs
use crate::{DcsvResult, Value};

pub trait VCont {
    /// Create empty virtual container
    fn new() -> Self;

    /// Move a given row to a target row index
    fn move_row(&mut self, src_index: usize, target_index: usize) -> DcsvResult<()>;

    /// Move a given column to target column index
    fn move_column(&mut self, src_index: usize, target_index: usize) -> DcsvResult<()>;

    /// Rename a column
    fn rename_column(&mut self, column_index: usize, new_name: &str) -> DcsvResult<()>;

    /// Set values to a column
    fn set_column(&mut self, column_index: usize, value: Value) -> DcsvResult<()>;

    /// Edit a row
    fn edit_row(&mut self, row_index: usize, values: &[Option<Value>]) -> DcsvResult<()>;

    /// Set values to a row
    ///
    /// This assumes that given values accord to column's order.
    fn set_row(&mut self, row_index: usize, values: &[Value]) -> DcsvResult<()>;

    /// get cell data by coordinate
    fn get_cell(&self, x: usize, y: usize) -> Option<&Value>;

    /// Set cell value by coordinate
    fn set_cell(&mut self, x: usize, y: usize, value: Value) -> DcsvResult<()>;

    /// Insert a row to given index
    fn insert_row(&mut self, row_index: usize, source: Option<&[Value]>) -> DcsvResult<()>;

    /// Delete a row with given row_index
    fn delete_row(&mut self, row_index: usize) -> bool;

    /// Insert a column with given column informations
    fn insert_column(&mut self, column_index: usize, column_name: &str) -> DcsvResult<()>;

    /// Delete a column with given column index
    fn delete_column(&mut self, column_index: usize) -> DcsvResult<()>;

    /// Get total rows count
    fn get_row_count(&self) -> usize;

    /// Get total columns count
    fn get_column_count(&self) -> usize;

    /// Drop all data from virtual data
    fn drop_data(&mut self);
}
