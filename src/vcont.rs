//! VCont is a generic trait for various virtual csv structs

use crate::{DcsvResult, Value};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CellAlignType {
    None,
    Left,
    Center,
    Right,
}

/// Generic trait over both virtual_data and virtual_array
///
/// This provides some genral methods over csv value manipulation
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

    /// Apply closure to all values
    fn apply_all<F: FnMut(&mut Value)>(&mut self, f: F);

    /// Fully iterate cells to update max_width
    fn update_width_global(&mut self);

    /// Get aligned string table
    fn get_formatted_string(&self, line_delimiter: &str, align_type: CellAlignType) -> String;

    /// Get table as raw string vectors of vectors
    fn get_string_table(&self, align_type: CellAlignType) -> Vec<Vec<String>>;
}
