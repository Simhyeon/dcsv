//! Virtual array module

use unicode_width::UnicodeWidthStr;

use crate::{meta::Meta, vcont::VCont, Column, DcsvError, DcsvResult, Value};
use std::cmp::Ordering;

/// Virtual array which contains csv information in a form of arrays.
///
/// - VirtualArray holds row information as vectors. Therefore indexing is generally faster than virtual data struct.
/// - VirtualArray allows duplicate columns
/// - VirtualArray doesn't allow limiters.
#[derive(Clone)]
pub struct VirtualArray {
    pub metas: Vec<Meta>,
    pub columns: Vec<Column>,
    pub rows: Vec<Vec<Value>>,
}

impl Default for VirtualArray {
    fn default() -> Self {
        Self::new()
    }
}

impl VCont for VirtualArray {
    fn new() -> Self {
        Self {
            metas: vec![],
            columns: vec![],
            rows: vec![],
        }
    }

    fn get_row_count(&self) -> usize {
        self.rows.len()
    }

    fn get_column_count(&self) -> usize {
        self.columns.len()
    }

    fn drop_data(&mut self) {
        self.columns.clear();
        self.rows.clear();
    }

    /// Rename a column
    ///
    /// This will simply change the name of the column and doesn't affect rows.
    fn rename_column(&mut self, column_index: usize, new_name: &str) -> DcsvResult<()> {
        self.columns[column_index].name = new_name.to_owned();
        Ok(())
    }

    /// Set values to a column
    ///
    /// Given value will override every row's value
    fn set_column(&mut self, column_index: usize, value: Value) -> DcsvResult<()> {
        if !self.is_valid_cell_coordinate(0, column_index) {
            return Err(DcsvError::OutOfRangeError);
        }

        for row in &mut self.rows {
            self.metas[column_index].update_width_from_value(&value);
            row[column_index] = value.clone();
        }
        Ok(())
    }

    /// Edit a row
    ///
    /// Only edit row's cell when value is not none
    fn edit_row(&mut self, row_index: usize, values: &[Option<Value>]) -> DcsvResult<()> {
        // Row's value doesn't match length of columns
        if values.len() != self.get_column_count() {
            return Err(DcsvError::InsufficientRowData);
        }

        // Invalid cooridnate
        if !self.is_valid_cell_coordinate(row_index, 0) {
            return Err(DcsvError::OutOfRangeError);
        }

        // It is safe to unwrap because row_number
        // was validated by is_valid_cell_coordinate method.
        let row = &mut self.rows[row_index];
        for (idx, v) in values.iter().enumerate() {
            if let Some(new_value) = v {
                self.metas[idx].update_width_from_value(new_value);
                row[idx] = new_value.clone();
            }
        }

        Ok(())
    }

    /// Insert a row to given index
    ///
    /// This can yield out of range error
    fn insert_row(&mut self, row_index: usize, source: Option<&[Value]>) -> DcsvResult<()> {
        if row_index > self.get_row_count() {
            return Err(DcsvError::InvalidColumn(format!(
                "Cannot add row to out of range position : {}",
                row_index
            )));
        }
        if let Some(source) = source {
            self.check_row_length(source)?;
            self.rows.insert(row_index, source.to_vec());
        } else {
            let row = vec![Value::Text(String::new()); self.columns.len()];
            self.rows.insert(row_index, row);
        }
        for (col, value) in self.metas.iter_mut().zip(self.rows[row_index].iter()) {
            col.update_width_from_value(value)
        }
        Ok(())
    }

    /// Delete a row with given row_index
    ///
    /// This doesn't fail but silently do nothing if index is out of range
    fn delete_row(&mut self, row_index: usize) -> bool {
        let row_count = self.get_row_count();
        if row_count == 0 || row_count < row_index {
            return false;
        }
        let removed = self.rows.remove(row_index);

        let to_be_updated_colum_index = removed
            .iter()
            .enumerate()
            .zip(self.metas.iter_mut())
            .filter_map(|((idx, item), meta)| {
                if item.get_width() >= meta.max_unicode_width {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        // TODO
        // It is safely to unwrap because column is already confirmed to exist
        for idx in to_be_updated_colum_index {
            // self.rows[idx]
            let mut new_max = 0;
            for cell in self.get_column_iterator(idx).expect("This should not fail") {
                new_max = new_max.max(cell.get_width());
            }
            self.metas[idx].set_width(new_max);
        }
        true
    }

    /// Insert a column with given column informations
    ///
    /// * column_index  : Position to put column
    /// * column_name   : New column name
    fn insert_column(&mut self, column_index: usize, column_name: &str) -> DcsvResult<()> {
        if column_index > self.get_column_count() {
            return Err(DcsvError::InvalidColumn(format!(
                "Cannot add column to out of range position : {}",
                column_index
            )));
        }

        let mut meta = Meta::new();
        meta.set_width(UnicodeWidthStr::width(column_name));
        self.metas.insert(column_index, meta);
        self.columns
            .insert(column_index, Column::empty(column_name));
        for row in &mut self.rows {
            row.insert(column_index, Value::Text(String::new()));
        }
        Ok(())
    }

    /// Delete a column with given column index
    fn delete_column(&mut self, column_index: usize) -> DcsvResult<()> {
        if !self.is_valid_cell_coordinate(0, column_index) {
            return Err(DcsvError::OutOfRangeError);
        }

        for row in &mut self.rows {
            row.remove(column_index);
        }

        self.metas.remove(column_index);
        self.columns.remove(column_index);

        // If column is empty, drop all rows
        if self.get_column_count() == 0 {
            self.rows = vec![];
        }

        Ok(())
    }

    fn get_cell(&self, x: usize, y: usize) -> Option<&Value> {
        if !self.is_valid_cell_coordinate(x, y) {
            return None;
        }
        let value = &self.rows[x][y];

        Some(value)
    }

    fn set_cell(&mut self, x: usize, y: usize, value: Value) -> DcsvResult<()> {
        if !self.is_valid_cell_coordinate(x, y) {
            return Err(DcsvError::OutOfRangeError);
        }
        self.metas[y].update_width_from_value(&value);
        self.rows[x][y] = value;
        Ok(())
    }

    /// Move a given column to target column index
    fn move_column(&mut self, src_index: usize, target_index: usize) -> DcsvResult<()> {
        let column_count = self.get_column_count();
        if src_index >= column_count || target_index >= column_count {
            return Err(DcsvError::OutOfRangeError);
        }

        let move_direction = src_index.cmp(&target_index);
        match move_direction {
            // Go left
            Ordering::Greater => {
                let mut index = src_index;
                let mut next = index - 1;
                while next >= target_index {
                    self.columns.swap(index, next);
                    self.metas.swap(index, next);

                    // Usize specific check code
                    if next == 0 {
                        break;
                    }

                    // Update index values
                    index -= 1;
                    next -= 1;
                }
            }
            Ordering::Less => {
                // Go right
                let mut index = src_index;
                let mut next = index + 1;
                while next <= target_index {
                    self.columns.swap(index, next);
                    self.metas.swap(index, next);

                    // Update index values
                    index += 1;
                    next += 1;
                }
            }
            Ordering::Equal => (),
        }
        Ok(())
    }

    fn move_row(&mut self, src_index: usize, target_index: usize) -> DcsvResult<()> {
        let row_count = self.get_row_count();
        if src_index >= row_count || target_index >= row_count {
            return Err(DcsvError::OutOfRangeError);
        }

        let move_direction = src_index.cmp(&target_index);
        match move_direction {
            // Go left
            Ordering::Greater => {
                let mut index = src_index;
                let mut next = index - 1;
                while next >= target_index {
                    self.rows.swap(index, next);

                    // Usize specific check code
                    if next == 0 {
                        break;
                    }

                    // Update index values
                    index -= 1;
                    next -= 1;
                }
            }
            Ordering::Less => {
                // Go right
                let mut index = src_index;
                let mut next = index + 1;
                while next <= target_index {
                    self.rows.swap(index, next);

                    // Update index values
                    index += 1;
                    next += 1;
                }
            }
            Ordering::Equal => (),
        }
        Ok(())
    }

    fn set_row(&mut self, row_index: usize, values: &[Value]) -> DcsvResult<()> {
        // Row's value doesn't match length of columns
        if values.len() != self.get_column_count() {
            return Err(DcsvError::InsufficientRowData);
        }
        // Invalid cooridnate
        if !self.is_valid_cell_coordinate(row_index, 0) {
            return Err(DcsvError::OutOfRangeError);
        }
        for (idx, v) in values.iter().enumerate() {
            self.metas[idx].update_width_from_value(v);
        }
        self.rows[row_index] = values.to_vec();

        Ok(())
    }

    fn apply_all<F: FnMut(&mut Value)>(&mut self, mut f: F) {
        for row in &mut self.rows {
            for value in row {
                f(value)
            }
        }
    }

    fn update_width_global(&mut self) {
        // Row iterate
        for idx in 0..self.get_row_count() {
            // Column iterate
            for cidx in 0..self.get_column_count() {
                let width = self.get_cell(idx, cidx).unwrap().get_width();
                self.metas[cidx].update_width(width);
            }
        }
    }
}

impl VirtualArray {
    /// Get iterator of a column with given index
    pub fn get_column_iterator(
        &self,
        column_index: usize,
    ) -> DcsvResult<std::vec::IntoIter<&Value>> {
        if self.columns.len() <= column_index {
            return Err(DcsvError::OutOfRangeError);
        }
        Ok(self
            .rows
            .iter()
            .map(|s| s.get(column_index).unwrap())
            .collect::<Vec<_>>()
            .into_iter())
    }

    /// Check if cell coordinate is not out of range
    fn is_valid_cell_coordinate(&self, x: usize, y: usize) -> bool {
        if x < self.get_row_count() && y < self.get_column_count() {
            return true;
        }

        false
    }

    /// Check if given values' length matches column's legnth
    fn check_row_length(&self, values: &[Value]) -> DcsvResult<()> {
        match self.get_column_count().cmp(&values.len()) {
            Ordering::Equal => (),
            Ordering::Less | Ordering::Greater => {
                return Err(DcsvError::InvalidRowData(format!(
                    r#"Given row length is "{}" while columns length is "{}""#,
                    values.len(),
                    self.get_column_count()
                )))
            }
        }
        Ok(())
    }
}

/// to_string implementation for virtual array
///
/// This returns csv value string
impl std::fmt::Display for VirtualArray {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut csv_src = String::new();
        let column_row = self
            .columns
            .iter()
            .map(|s| s.name.as_str())
            .collect::<Vec<_>>()
            .join(",")
            + "\n";
        csv_src.push_str(&column_row);

        let rows = self
            .rows
            .iter()
            .map(|row| {
                row.iter()
                    .map(|row| row.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            })
            .collect::<Vec<_>>()
            .join("\n");
        csv_src.push_str(&rows);
        write!(f, "{}", csv_src)
    }
}
