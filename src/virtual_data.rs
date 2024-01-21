//! Virtual data module

use unicode_width::UnicodeWidthStr;

use crate::error::{DcsvError, DcsvResult};
use crate::meta::Meta;
use crate::value::{Value, ValueLimiter, ValueType};
use crate::vcont::VCont;
use crate::CellAlignType;
use std::cmp::Ordering;
use std::collections::HashMap;

/// Header for csv schema
pub const SCHEMA_HEADER: &str = "column,type,default,variant,pattern";

/// Virtual data struct which contains csv information
///
/// - VirtualData holds row information as hashmap. Therefore modifying data( cell, row or column ) is generally faster than virtual array struct.
/// - VirtualData cannot have duplicate column name due to previous hashmap implementaiton
/// - VirtualData allows limiters to confine csv value's possible states.
#[derive(Clone)]
pub struct VirtualData {
    pub metas: Vec<Meta>,
    pub columns: Vec<Column>,
    pub rows: Vec<Row>,
}

impl Default for VirtualData {
    fn default() -> Self {
        Self::new()
    }
}

impl VCont for VirtualData {
    /// Create empty virtual data
    fn new() -> Self {
        Self {
            metas: vec![],
            columns: vec![],
            rows: vec![],
        }
    }

    /// Move given row to a target row index
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

    /// Rename a column
    ///
    /// Column's name cannot be an exsiting name
    ///
    /// * column   : column_index
    /// * new_name : New column name
    fn rename_column(&mut self, column_index: usize, new_name: &str) -> DcsvResult<()> {
        let next_column_index = self.try_get_column_index(new_name);

        if !self.is_valid_cell_coordinate(0, column_index) {
            return Err(DcsvError::OutOfRangeError);
        }

        if next_column_index.is_some() {
            return Err(DcsvError::InvalidColumn(format!(
                "Cannot rename to \"{}\" which already exists",
                &new_name
            )));
        }

        let previous = self.columns[column_index].rename(new_name);
        for row in &mut self.rows {
            row.rename_column(&previous, new_name);
        }
        Ok(())
    }

    /// Set values to a column
    ///
    /// Given value will override every row's value
    fn set_column(&mut self, column_index: usize, value: Value) -> DcsvResult<()> {
        if !self.is_valid_cell_coordinate(0, column_index) {
            return Err(DcsvError::OutOfRangeError);
        }

        let column = &self.columns[column_index].name;
        let col_meta = &mut self.metas[column_index];

        for row in &mut self.rows {
            col_meta.update_width_from_value(&value);
            row.update_cell_value(column, value.clone());
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

        let col_value_iter = self.columns.iter().enumerate().zip(values.iter());

        for ((_, col), value) in col_value_iter.clone() {
            if let Some(value) = value {
                // Early return if doesn't qualify a single element
                if !col.limiter.qualify(value) {
                    return Err(DcsvError::InvalidRowData(format!(
                        "\"{}\" doesn't qualify \"{}\"'s limiter",
                        value, col.name
                    )));
                }
            }
        }

        // It is safe to unwrap because row_number
        // was validated by is_valid_cell_coordinate method.
        let row = self.rows.get_mut(row_index).unwrap();
        for ((idx, col), value) in col_value_iter {
            if let Some(value) = value {
                self.metas[idx].update_width_from_value(value);
                row.update_cell_value(&col.name, value.clone())
            }
        }

        Ok(())
    }

    // TODO
    // 1. Check limiter
    // 2. Check if value exists
    /// Set values to a row
    ///
    /// This assumes that given values accord to column's order.
    /// This method will fail when given value fails to qualify column's limiter.
    fn set_row(&mut self, row_index: usize, values: &[Value]) -> DcsvResult<()> {
        // Row's value doesn't match length of columns
        if values.len() != self.get_column_count() {
            return Err(DcsvError::InsufficientRowData);
        }
        // Invalid cooridnate
        if !self.is_valid_cell_coordinate(row_index, 0) {
            return Err(DcsvError::OutOfRangeError);
        }

        let col_value_iter = self.columns.iter().enumerate().zip(values.iter());

        for ((_, col), value) in col_value_iter.clone() {
            // Early return if doesn't qualify a single element
            if !col.limiter.qualify(value) {
                return Err(DcsvError::InvalidRowData(format!(
                    "\"{}\" doesn't qualify \"{}\"'s limiter",
                    value, col.name
                )));
            }
        }

        // It is safe to unwrap because row_number
        // was validated by is_valid_cell_coordinate method.
        let row = self.rows.get_mut(row_index).unwrap();
        for ((idx, col), value) in col_value_iter {
            self.metas[idx].update_width_from_value(value);
            row.update_cell_value(&col.name, value.clone());
        }

        Ok(())
    }

    /// get cell data by coordinate
    fn get_cell(&self, x: usize, y: usize) -> Option<&Value> {
        if let Ok(column) = self.get_column_if_valid(x, y) {
            self.rows[x].get_cell_value(&column.name)
        } else {
            None
        }
    }

    /// Set cell value by coordinate
    fn set_cell(&mut self, x: usize, y: usize, value: Value) -> DcsvResult<()> {
        let name = self.get_column_if_valid(x, y)?.name.to_owned();

        self.is_valid_column_data(y, &value)?;
        self.metas[y].update_width_from_value(&value);
        self.rows[x].update_cell_value(&name, value);

        Ok(())
    }

    // THis should insert row with given column limiters
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
        let mut new_row = Row::new();
        if let Some(source) = source {
            self.check_row_length(source)?;
            let iter = self.columns.iter().zip(source.iter());

            for (col, value) in iter.clone() {
                if !col.limiter.qualify(value) {
                    return Err(DcsvError::InvalidRowData(format!(
                        "\"{}\" doesn't qualify \"{}\"'s limiter",
                        value, col.name
                    )));
                }
            }

            iter.for_each(|(col, v)| new_row.insert_cell(&col.name, v.clone()));
        } else {
            for col in &self.columns {
                new_row.insert_cell(&col.name, col.get_default_value());
            }
        }
        for (col, value) in self
            .metas
            .iter_mut()
            .zip(new_row.to_vector(&self.columns)?.iter())
        {
            col.update_width_from_value(value)
        }
        self.rows.insert(row_index, new_row);
        Ok(())
    }

    fn insert_column(&mut self, column_index: usize, column_name: &str) -> DcsvResult<()> {
        if column_index > self.get_column_count() {
            return Err(DcsvError::InvalidColumn(format!(
                "Cannot add column to out of range position : {}",
                column_index
            )));
        }
        if self.try_get_column_index(column_name).is_some() {
            return Err(DcsvError::InvalidColumn(format!(
                "Cannot add existing column = \"{}\"",
                column_name
            )));
        }
        let new_column = Column::new(column_name, ValueType::Text, None);
        let default_value = new_column.get_default_value();
        for row in &mut self.rows {
            row.insert_cell(&new_column.name, default_value.clone());
        }

        let mut meta = Meta::new();
        let max_width = UnicodeWidthStr::width(column_name).max(default_value.get_width());
        meta.set_width(max_width);
        self.metas.insert(column_index, meta);
        self.columns.insert(column_index, new_column);
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
            .get_iterator(&self.columns)
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

        // It is safely to unwrap because column is already confirmed to exist
        for idx in to_be_updated_colum_index {
            let mut new_max = 0;
            for cell in self.get_column_iterator(idx).expect("This should not fail") {
                new_max = new_max.max(cell.get_width());
            }
            self.metas[idx].set_width(new_max);
        }

        true
    }

    /// Delete a column with given column index
    fn delete_column(&mut self, column_index: usize) -> DcsvResult<()> {
        let name = self.get_column_if_valid(0, column_index)?.name.to_owned();

        for row in &mut self.rows {
            row.remove_cell(&name);
        }

        self.metas.remove(column_index);
        self.columns.remove(column_index);

        // If column is empty, drop all rows
        if self.get_column_count() == 0 {
            self.rows = vec![];
        }

        Ok(())
    }

    /// Get total rows count
    fn get_row_count(&self) -> usize {
        self.rows.len()
    }

    /// Get total columns count
    fn get_column_count(&self) -> usize {
        self.columns.len()
    }

    /// Drop all data from virtual data
    fn drop_data(&mut self) {
        self.columns.clear();
        self.rows.clear();
    }

    /// Apply closure to all values
    fn apply_all<F: FnMut(&mut Value)>(&mut self, mut f: F) {
        for row in &mut self.rows {
            for value in row.values.values_mut() {
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

    fn get_formatted_string(&self, line_delimiter: &str, align_type: CellAlignType) -> String {
        let table = self.get_string_table(align_type);
        let mut formatted = String::new();
        let mut iter = table.iter().peekable();
        while let Some(item) = iter.next() {
            formatted.push_str(&item.join(" "));
            if iter.peek().is_some() {
                formatted.push_str(line_delimiter);
            }
        }

        formatted
    }

    fn get_string_table(&self, align_type: crate::CellAlignType) -> Vec<Vec<String>> {
        // Currently only left align
        #[inline]
        fn pad(target: &str, max_width: usize, align_type: CellAlignType) -> String {
            if align_type == CellAlignType::None {
                return target.to_string();
            }
            let t_len = unicode_width::UnicodeWidthStr::width(target);
            if t_len > max_width {
                panic!(
                    "This is a critical logic error and should not happen on sound code production"
                );
            }

            match align_type {
                CellAlignType::Left => format!("{0}{1}", target, " ".repeat(max_width - t_len)),
                CellAlignType::Right => format!("{1}{0}", target, " ".repeat(max_width - t_len)),
                CellAlignType::Center => {
                    let leading = ((max_width - t_len) as f32 / 2.0).ceil() as usize;
                    let following = max_width - t_len - leading;
                    format!(
                        "{1}{0}{2}",
                        target,
                        " ".repeat(leading),
                        " ".repeat(following)
                    )
                }
                _ => unreachable!(),
            }
        }

        let mut formatted = vec![];
        let width_vector = self
            .columns
            .iter()
            .zip(self.metas.iter())
            .map(|(col, meta)| {
                UnicodeWidthStr::width(col.name.as_str()).max(meta.max_unicode_width)
            })
            .collect::<Vec<_>>();

        let column_row = self
            .columns
            .iter()
            .zip(width_vector.iter())
            .map(|(c, w)| pad(c.name.as_str(), *w, align_type))
            .collect::<Vec<String>>();
        formatted.push(column_row);

        let columns = self
            .columns
            .iter()
            .zip(width_vector.iter())
            .map(|(col, width)| (col.name.as_str(), *width))
            .collect::<Vec<(&str, usize)>>();

        for row in self.rows.iter() {
            let row_value = columns
                .iter()
                .map(|(col_name, width)| {
                    pad(
                        &row.get_cell_value(col_name)
                            .unwrap_or(&Value::Text(String::new()))
                            .to_string(),
                        *width,
                        align_type,
                    )
                })
                .collect::<Vec<String>>();

            formatted.push(row_value);
        }
        formatted
    }
}

impl VirtualData {
    /// Get read only data from virtual data
    ///
    /// This clones every value into a ReadOnlyData.
    /// If the purpose is to simply iterate over values, prefer read_only_ref method.
    pub fn read_only(&self) -> ReadOnlyData {
        ReadOnlyData::from(self)
    }

    /// Get read only data from virtual data, but as reference
    pub fn read_only_ref(&self) -> ReadOnlyDataRef {
        ReadOnlyDataRef::from(self)
    }

    /// Set cell's value with given string value
    ///
    /// This will fail if the value cannot be converted to column's type
    pub fn set_cell_from_string(&mut self, x: usize, y: usize, value: &str) -> DcsvResult<()> {
        let key_column = self.get_column_if_valid(x, y)?;
        let nvalue = match key_column.column_type {
            ValueType::Text => Value::Text(value.to_string()),
            ValueType::Number => Value::Number(value.parse().map_err(|_| {
                DcsvError::InvalidCellData(format!(
                    "Given value is \"{}\" which is not a number",
                    value
                ))
            })?),
        };

        self.metas[y].update_width_from_value(&nvalue);
        self.set_cell(x, y, nvalue)?;

        Ok(())
    }

    /// Insert a column with given column informations
    ///
    /// # Args
    ///
    /// * column_index  : Position to put column
    /// * column_name   : New column name
    /// * column_type   : Column's type
    /// * limiter       : Set limiter with
    /// * placeholder   : Placeholder will be applied to every row
    pub fn insert_column_with_type(
        &mut self,
        column_index: usize,
        column_name: &str,
        column_type: ValueType,
        limiter: Option<ValueLimiter>,
        placeholder: Option<Value>,
    ) -> DcsvResult<()> {
        if column_index > self.get_column_count() {
            return Err(DcsvError::InvalidColumn(format!(
                "Cannot add column to out of range position : {}",
                column_index
            )));
        }
        if self.try_get_column_index(column_name).is_some() {
            return Err(DcsvError::InvalidColumn(format!(
                "Cannot add existing column = \"{}\"",
                column_name
            )));
        }
        let new_column = Column::new(column_name, column_type, limiter);
        let default_value = new_column.get_default_value();
        let value = placeholder.unwrap_or(default_value.clone());
        for row in &mut self.rows {
            row.insert_cell(&new_column.name, value.clone());
        }
        self.columns.insert(column_index, new_column);

        let mut meta = Meta::new();
        let max_width = UnicodeWidthStr::width(column_name).max(default_value.get_width());
        meta.set_width(max_width);
        self.metas.insert(column_index, meta);
        Ok(())
    }

    /// Set a limiter to a column
    ///
    /// # Args
    ///
    /// * column  : column's index
    /// * limiter : Target limiter
    /// * panic   : If true, failed set will occur panic
    pub fn set_limiter(
        &mut self,
        column: usize,
        limiter: &ValueLimiter,
        panic: bool,
    ) -> DcsvResult<()> {
        let column = &mut self.columns[column];
        for (index, row) in self.rows.iter_mut().enumerate() {
            let mut qualified = true;
            let mut converted = None;
            let mut convert_to = None;
            if let Some(value) = row.get_cell_value(&column.name) {
                // Check if value can be converted at most
                if let Some(ttype) = limiter.is_convertible(value) {
                    converted.replace(Value::from_str(&value.to_string(), ttype)?);
                    convert_to = Some(ttype);
                }

                // Check if value qualify limiter condition
                if !limiter.qualify(converted.as_ref().unwrap_or(value)) {
                    qualified = false;
                    convert_to = None;
                    if panic {
                        return Err(DcsvError::InvalidCellData(format!(
                            "Cell {},{} doesn't match limiter's qualification",
                            index, column.name
                        )));
                    }
                }
            } else {
                return Err(DcsvError::InvalidRowData(
                    "Failed to get row data while setting limiter".to_string(),
                ));
            }

            if let Some(ttype) = convert_to {
                row.change_cell_type(&column.name, ttype)?;
            } else if !qualified && !panic {
                // Force update to defualt value
                // It is mostly safe to unwrap because default is required for pattern or variant
                // but, limiter might only have a single "type" value
                row.update_cell_value(
                    &column.name,
                    limiter
                        .get_default()
                        .unwrap_or(&Value::empty(limiter.get_type()))
                        .clone(),
                );
            }
        }
        column.set_limiter(limiter.clone());
        Ok(())
    }

    /// Qualify data and get reference of qualifed rows.
    pub fn qualify(&self, column: usize, limiter: &ValueLimiter) -> DcsvResult<Vec<&Row>> {
        let mut rows = vec![];
        let column = &self.columns[column];
        for row in &self.rows {
            if let Some(value) = row.get_cell_value(&column.name) {
                // Check if value qualify limiter condition
                if limiter.qualify(value) {
                    rows.push(row)
                }
            } else {
                return Err(DcsvError::InvalidRowData(
                    "Failed to get row data while qualifying".to_string(),
                ));
            }
        }
        Ok(rows)
    }

    /// Qualify data with multiple limiters and get reference of qualifed rows.
    pub fn qualify_multiple(
        &self,
        qualifiers: Vec<(usize, &ValueLimiter)>,
    ) -> DcsvResult<Vec<&Row>> {
        let mut rows = vec![];
        // Rows loop
        'outer: for row in &self.rows {
            // Values loop in
            for (column, limiter) in &qualifiers {
                let column = &self.columns[*column];
                if let Some(value) = row.get_cell_value(&column.name) {
                    // Check if value qualify limiter condition
                    if !limiter.qualify(value) {
                        continue 'outer;
                    }
                } else {
                    return Err(DcsvError::InvalidRowData(
                        "Failed to get row data while qualifying".to_string(),
                    ));
                }
            }

            // Only push if all qualifiers suceeded.
            rows.push(row);
        }
        Ok(rows)
    }

    /// Export virtual data's schema(limiter) as string form
    ///
    /// Schema is expressed as csv value. Each line is structured with following order.
    ///
    /// - column
    /// - type
    /// - default
    /// - variant
    /// - pattern
    pub fn export_schema(&self) -> String {
        let mut schema = format!("{}\n", SCHEMA_HEADER);
        for col in &self.columns {
            let mut line = col.name.clone() + ",";
            let limiter = &col.limiter;
            line.push_str(&limiter.get_type().to_string());
            line.push(',');
            line.push_str(
                &limiter
                    .get_default()
                    .map(|s| s.to_string())
                    .unwrap_or_default(),
            );
            line.push(',');
            line.push_str(
                &limiter
                    .get_variant()
                    .map(|s| s.iter().map(|s| s.to_string()).collect::<Vec<String>>())
                    .unwrap_or_default()
                    .join(" "),
            );
            line.push(',');
            line.push_str(
                &limiter
                    .get_pattern()
                    .map(|s| s.to_string())
                    .unwrap_or_default(),
            );

            schema.push_str(&(line + "\n"));
        }
        schema
    }

    // <DRY>
    /// Get a column index from src
    ///
    /// Src can be either colum name or column index
    /// If colum index is out of range, it returns none
    pub fn try_get_column_index(&self, src: &str) -> Option<usize> {
        let column_index = match src.parse::<usize>() {
            Err(_) => self.columns.iter().position(|c| c.name == src),
            Ok(index) => {
                if index < self.get_column_count() {
                    Some(index)
                } else {
                    None
                }
            }
        };
        column_index
    }

    /// Check if cell coordinate is not out of range
    ///
    /// # Return
    ///
    /// True if given coordinate is within data's boundary, false if not.
    fn is_valid_cell_coordinate(&self, x: usize, y: usize) -> bool {
        if x < self.get_row_count() && y < self.get_column_count() {
            return true;
        }

        false
    }

    /// Check if given coordinate exits and return target column
    fn get_column_if_valid(&self, x: usize, y: usize) -> DcsvResult<&Column> {
        if !self.is_valid_cell_coordinate(x, y) {
            return Err(DcsvError::OutOfRangeError);
        }
        // It is sfe to uwnrap because
        // it was validated by prior is_valid_cell_coordinate method
        let key_column = self.columns.get(y).unwrap();
        Ok(key_column)
    }

    /// Check if given value corresponds to column limiter
    fn is_valid_column_data(&self, column: usize, value: &Value) -> DcsvResult<()> {
        if let Some(col) = self.columns.get(column) {
            if col.limiter.qualify(value) {
                Ok(())
            } else {
                Err(DcsvError::InvalidCellData(
                    "Given cell data failed to match limiter's restriction".to_string(),
                ))
            }
        } else {
            Err(DcsvError::InvalidRowData(format!(
                "Given column index \"{}\" doesn't exist",
                column
            )))
        }
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

    /// Get iterator.
    ///
    /// This methods returns iterator which respects column orders
    pub fn get_iterator(&self) -> std::vec::IntoIter<&Value> {
        let columns = &self.columns;
        let mut iterate = vec![];
        for row in &self.rows {
            iterate.extend(row.get_iterator(columns));
        }
        iterate.into_iter()
    }

    /// Get iterator of a column with given index
    pub fn get_column_iterator(
        &self,
        column_index: usize,
    ) -> DcsvResult<std::vec::IntoIter<&Value>> {
        let column = &self
            .columns
            .get(column_index)
            .ok_or_else(|| DcsvError::OutOfRangeError)?;
        let acc = (0..self.get_row_count())
            .filter_map(|idx| self.rows[idx].get_cell_value(&column.name))
            .collect::<Vec<_>>();
        Ok(acc.into_iter())
    }

    /// Get iterator of a row with given index
    ///
    /// This respects  columns orders
    pub fn get_row_iterator(&self, row_index: usize) -> DcsvResult<std::vec::IntoIter<&Value>> {
        let columns = &self.columns;
        Ok(self
            .rows
            .get(row_index)
            .ok_or(DcsvError::OutOfRangeError)?
            .get_iterator(columns))
    }
}

/// to_string implementation for virtual data
///
/// This returns csv value string
impl std::fmt::Display for VirtualData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut csv_src = String::new();
        let column_row = self
            .columns
            .iter()
            .map(|c| c.name.as_str())
            .collect::<Vec<&str>>()
            .join(",")
            + "\n";
        csv_src.push_str(&column_row);

        let columns = self
            .columns
            .iter()
            .map(|col| col.name.as_str())
            .collect::<Vec<&str>>();
        for row in &self.rows {
            let row_value = columns
                .iter()
                .map(|name| {
                    row.get_cell_value(name)
                        .unwrap_or(&Value::Text(String::new()))
                        .to_string()
                })
                .collect::<Vec<String>>()
                .join(",")
                + "\n";

            csv_src.push_str(&row_value);
        }
        // Remove trailing newline
        csv_src.pop();
        write!(f, "{}", csv_src)
    }
}

/// Column of virtual data
///
/// Column is "text" type by default but can be further configured with value limiter.
#[derive(Clone, Debug)]
pub struct Column {
    pub name: String,
    pub column_type: ValueType,
    pub limiter: ValueLimiter,
}

impl Column {
    /// Create empty column with name
    pub fn empty(name: &str) -> Self {
        Self {
            name: name.to_string(),
            column_type: ValueType::Text,
            limiter: ValueLimiter::default(),
        }
    }

    /// Create new column with properties
    pub fn new(name: &str, column_type: ValueType, limiter: Option<ValueLimiter>) -> Self {
        Self {
            name: name.to_string(),
            column_type,
            limiter: limiter.unwrap_or_default(),
        }
    }

    /// Get column name
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Get column type
    pub fn get_column_type(&self) -> &ValueType {
        &self.column_type
    }

    /// Rename a column and return an original name
    pub fn rename(&mut self, new_name: &str) -> String {
        std::mem::replace(&mut self.name, new_name.to_string())
    }

    /// Apply limiter to a column
    pub fn set_limiter(&mut self, limiter: ValueLimiter) {
        self.column_type = limiter.get_type();
        self.limiter = limiter;
    }

    /// Get default value by column
    ///
    /// Every value type has it's own default value.
    /// The default value can differ by limiter's variant of patterns and should comply to a
    /// limter's predicate.
    pub fn get_default_value(&self) -> Value {
        // has default
        if let Some(def) = self.limiter.get_default() {
            return def.clone();
        }

        // has variant
        let variant = self.limiter.get_variant();
        if let Some(vec) = variant {
            if !vec.is_empty() {
                return vec[0].clone();
            }
        }

        // Construct new default value
        match self.column_type {
            ValueType::Number => Value::Number(0),
            ValueType::Text => Value::Text(String::new()),
        }
    }
}

/// Row
///
/// Row is implementated as a hashmap. You cannot iterate row without column information.
#[derive(Clone, Debug)]
pub struct Row {
    pub values: HashMap<String, Value>,
}

impl Default for Row {
    fn default() -> Self {
        Self::new()
    }
}

impl Row {
    /// Create a new row
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    /// Convert row to vector with given columns
    ///
    /// It is totally valid to give partial columns into a row.
    pub fn to_vector(&self, columns: &[Column]) -> DcsvResult<Vec<&Value>> {
        let mut acc = Vec::default();
        for col in columns {
            acc.push(self.values.get(&col.name).ok_or_else(|| {
                DcsvError::InvalidColumn(
                    "Given column was not present thus cannot construct row vector".to_string(),
                )
            })?);
        }
        Ok(acc)
    }

    /// Get comma separated row string
    ///
    /// This requires columns because a row is not a linear container. Partial column is not an
    /// error but valid behaviour.
    pub fn to_string(&self, columns: &[Column]) -> DcsvResult<String> {
        let mut acc = String::new();
        for col in columns {
            acc.push_str(
                &self
                    .values
                    .get(&col.name)
                    .ok_or_else(|| {
                        DcsvError::InvalidColumn(
                            "Given column was not present thus cannot construct row string"
                                .to_string(),
                        )
                    })?
                    .to_string(),
            );
            acc.push(',');
        }
        acc.pop(); // Remove trailing comma
        Ok(acc)
    }

    /// Rename column name inside row map
    ///
    /// This doesn't validate column's name and should comply with column name rule to avoid
    /// unintended behaviour.
    pub fn rename_column(&mut self, name: &str, new_name: &str) {
        let previous = self.values.remove(name);

        if let Some(prev) = previous {
            self.values.insert(new_name.to_string(), prev);
        }
    }

    /// Insert a new cell(key, value pair) into a row
    pub fn insert_cell(&mut self, key: &str, value: Value) {
        self.values.insert(key.to_string(), value);
    }

    /// Get a cell value by a key
    pub fn get_cell_value(&self, key: &str) -> Option<&Value> {
        self.values.get(key)
    }

    /// Update a cell's value with a given value
    ///
    /// This doesn't fail and silently do nothing if key doesn't exist.
    pub fn update_cell_value(&mut self, key: &str, value: Value) {
        if let Some(v) = self.values.get_mut(key) {
            *v = value;
        }
    }

    /// Chagnes a cell's value type
    ///
    /// This method tries to naturally convert cell's type.
    /// Empty text value defaults to "0".
    pub fn change_cell_type(&mut self, key: &str, target_type: ValueType) -> DcsvResult<()> {
        if let Some(v) = self.values.get_mut(key) {
            match v {
                Value::Text(t) => {
                    if target_type == ValueType::Number {
                        // Empty text value can be evaluted to 0 value number
                        if t.is_empty() {
                            *v = Value::Number(0);
                            return Ok(());
                        }

                        *v = Value::Number(t.parse::<isize>().map_err(|_| {
                            DcsvError::InvalidCellData(format!(
                                "\"{}\" is not a valid value to be converted to type : \"{}\"",
                                t, target_type
                            ))
                        })?);
                    }
                }
                Value::Number(n) => {
                    if target_type == ValueType::Text {
                        *v = Value::Text(n.to_string());
                    }
                }
            }
        }
        Ok(())
    }

    /// Remove a cell by key
    pub fn remove_cell(&mut self, key: &str) {
        self.values.remove(key);
    }

    /// Get iterator with given columns
    pub fn get_iterator(&self, columns: &[Column]) -> std::vec::IntoIter<&Value> {
        let mut iterate = vec![];
        for col in columns {
            if let Some(value) = self.values.get(&col.name) {
                iterate.push(value);
            }
        }
        iterate.into_iter()
    }
}

/// Read only data
///
/// This is a cloned data from virtual_data, thus lifetime independent
#[derive(Debug)]
pub struct ReadOnlyData {
    pub columns: Vec<Column>,
    pub rows: Vec<Vec<Value>>,
}

impl From<&VirtualData> for ReadOnlyData {
    fn from(data: &VirtualData) -> Self {
        let mut rows: Vec<Vec<Value>> = vec![];
        for row in &data.rows {
            let mut static_row: Vec<Value> = vec![];
            for col in &data.columns {
                static_row.push(row.get_cell_value(&col.name).unwrap().clone())
            }
            rows.push(static_row);
        }
        Self {
            columns: data.columns.clone(),
            rows,
        }
    }
}

/// Borrowed read only data from virtual_data
///
/// * Columns : Vec<&str>
/// * rows    : Vec<Vec<&Value>>
#[derive(Debug)]
pub struct ReadOnlyDataRef<'data> {
    pub columns: Vec<&'data Column>,
    pub rows: Vec<Vec<&'data Value>>,
}

impl<'data> ReadOnlyDataRef<'data> {
    /// Get owned ReadOnlyData struct
    ///
    /// This clones all information into a separate struct
    pub fn to_owned(&self) -> ReadOnlyData {
        ReadOnlyData {
            columns: self.columns.iter().map(|&c| c.clone()).collect::<Vec<_>>(),
            rows: self
                .rows
                .iter()
                .map(|vv| vv.iter().map(|&v| v.clone()).collect::<Vec<_>>())
                .collect::<Vec<Vec<_>>>(),
        }
    }
}

impl<'data> From<&'data VirtualData> for ReadOnlyDataRef<'data> {
    fn from(data: &'data VirtualData) -> Self {
        let mut rows: Vec<Vec<&'data Value>> = vec![];
        for row in &data.rows {
            let mut static_row: Vec<&'data Value> = vec![];
            for col in &data.columns {
                static_row.push(row.get_cell_value(&col.name).unwrap())
            }
            rows.push(static_row);
        }
        Self {
            columns: data.columns.iter().collect(),
            rows,
        }
    }
}
