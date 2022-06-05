use crate::error::{DcsvError, DcsvResult};
use crate::value::{Value, ValueLimiter, ValueType};
use std::cmp::Ordering;
use std::collections::HashMap;

pub const SCHEMA_HEADER: &str = "column,type,default,variant,pattern";

#[derive(Clone)]
pub struct VirtualData {
    pub columns: Vec<Column>,
    pub rows: Vec<Row>,
}

impl VirtualData {
    pub fn new() -> Self {
        Self {
            columns: vec![],
            rows: vec![],
        }
    }

    /// Get readly only data from virtual data
    ///
    /// This clones every value into a ReadOnlyData. 
    /// If the purpose is to simply iterate over values, prefer read_only_ref method.
    pub fn read_only(&self) -> ReadOnlyData {
        ReadOnlyData::from(self)
    }

    /// Get readly only data from virtual data buf as reference
    pub fn read_only_ref(&self) -> ReadOnlyDataRef {
        ReadOnlyDataRef::from(self)
    }

    /// Set cell's value with given string value
    pub fn set_cell_from_string(&mut self, x: usize, y: usize, value: &str) -> DcsvResult<()> {
        let key_column = self.get_column_if_valid(x, y)?;
        match key_column.column_type {
            ValueType::Text => self.set_cell(x, y, Value::Text(value.to_string())),
            ValueType::Number => self.set_cell(
                x,
                y,
                Value::Number(value.parse().map_err(|_| {
                    DcsvError::InvalidCellData(format!(
                        "Given value is \"{}\" which is not a number",
                        value
                    ))
                })?),
            ),
        }
    }

    /// Move given row to target row number
    pub fn move_row(&mut self, src: usize, target: usize) -> DcsvResult<()> {
        let row_count = self.get_row_count();
        if src >= row_count || target >= row_count {
            return Err(DcsvError::OutOfRangeError);
        }

        let move_direction = src.cmp(&target);
        match move_direction {
            // Go left
            Ordering::Greater => {
                let mut index = src;
                let mut next = index - 1;
                while next >= target {
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
                let mut index = src;
                let mut next = index + 1;
                while next <= target {
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

    /// Move given column to target column number
    pub fn move_column(&mut self, src: usize, target: usize) -> DcsvResult<()> {
        let column_count = self.get_column_count();
        if src >= column_count || target >= column_count {
            return Err(DcsvError::OutOfRangeError);
        }

        let move_direction = src.cmp(&target);
        match move_direction {
            // Go left
            Ordering::Greater => {
                let mut index = src;
                let mut next = index - 1;
                while next >= target {
                    self.columns.swap(index, next);

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
                let mut index = src;
                let mut next = index + 1;
                while next <= target {
                    self.columns.swap(index, next);

                    // Update index values
                    index += 1;
                    next += 1;
                }
            }
            Ordering::Equal => (),
        }
        Ok(())
    }

    /// Rename column
    ///
    /// Column cannot be a number or exsiting one
    pub fn rename_column(&mut self, column: &str, new_name: &str) -> DcsvResult<()> {
        if let Ok(_) = new_name.parse::<f64>() {
            return Err(DcsvError::InvalidColumn(format!(
                "Given invalid column name, \"{new_name}\" which is a number"
            )));
        }
        let column_index = self.try_get_column_index(column);
        let next_column_index = self.try_get_column_index(new_name);

        if let None = column_index {
            return Err(DcsvError::OutOfRangeError);
        }

        if let Some(_) = next_column_index {
            return Err(DcsvError::InvalidColumn(format!(
                "Cannot rename to \"{}\" which already exists",
                &new_name
            )));
        }

        let previous = self.columns[column_index.unwrap()].rename(new_name);
        for row in &mut self.rows {
            row.rename_column(&previous, new_name);
        }
        Ok(())
    }

    // TODO
    // 1. Check limiter
    // 2. Check if value exists
    /// Set values to a column
    pub fn set_column(&mut self, column: &str, value: Value) -> DcsvResult<()> {
        let column_index = self.try_get_column_index(column);
        if let None = column_index {
            return Err(DcsvError::OutOfRangeError);
        }
        let column = &self.columns[column_index.unwrap()].name;
        for row in &mut self.rows {
            row.update_cell_value(column, value.clone());
        }
        Ok(())
    }

    /// Edit a row
    ///
    /// Only edit row's cell when value is not none
    pub fn edit_row(
        &mut self,
        row_number: usize,
        mut values: Vec<Option<Value>>,
    ) -> DcsvResult<()> {
        // Row's value doesn't match length of columns
        if values.len() != self.get_column_count() {
            return Err(DcsvError::InsufficientRowData);
        }
        // Invalid cooridnate
        if !self.is_valid_cell_coordinate(row_number, 0) {
            return Err(DcsvError::OutOfRangeError);
        }

        let col_value_iter = self.columns.iter().zip(values.iter());

        for (col, value) in col_value_iter {
            if let Some(value) = value {
                // Early return if doesn't qualify a single element
                if !col.limiter.qualify(value) {
                    return Err(DcsvError::InvalidRowData(format!(
                        "\"{}\" doesn't qualify \"{}\"'s limiter",
                        value.to_string(),
                        col.name
                    )));
                }
            }
        }
        let col_value_iter = self.columns.iter().zip(values.iter_mut());

        // It is safe to unwrap because row_number
        // was validated by is_valid_cell_coordinate method.
        let row = self.rows.get_mut(row_number).unwrap();
        for (col, value) in col_value_iter {
            if let Some(value) = value {
                row.update_cell_value(&col.name, std::mem::replace(value, Value::default()))
            }
        }

        Ok(())
    }

    // TODO
    // 1. Check limiter
    // 2. Check if value exists
    /// Set values to a row
    pub fn set_row(&mut self, row_number: usize, values: Vec<Value>) -> DcsvResult<()> {
        // Row's value doesn't match length of columns
        if values.len() != self.get_column_count() {
            return Err(DcsvError::InsufficientRowData);
        }
        // Invalid cooridnate
        if !self.is_valid_cell_coordinate(row_number, 0) {
            return Err(DcsvError::OutOfRangeError);
        }

        let col_value_iter = self.columns.iter().zip(values.iter());

        for (col, value) in col_value_iter.clone() {
            // Early return if doesn't qualify a single element
            if !col.limiter.qualify(value) {
                return Err(DcsvError::InvalidRowData(format!(
                    "\"{}\" doesn't qualify \"{}\"'s limiter",
                    value.to_string(),
                    col.name
                )));
            }
        }

        // It is safe to unwrap because row_number
        // was validated by is_valid_cell_coordinate method.
        let row = self.rows.get_mut(row_number).unwrap();
        for (col, value) in col_value_iter {
            row.update_cell_value(&col.name, value.clone())
        }

        Ok(())
    }

    /// get cell data by coordinate
    pub fn get_cell(&self, x: usize, y: usize) -> DcsvResult<Option<&Value>> {
        let name = self.get_column_if_valid(x, y)?.name.to_owned();
        let value = self.rows[x].get_cell_value(&name);

        Ok(value)
    }

    /// Set cell value by coordinate
    pub fn set_cell(&mut self, x: usize, y: usize, value: Value) -> DcsvResult<()> {
        let name = self.get_column_if_valid(x, y)?.name.to_owned();

        self.is_valid_column_data(y, &value)?;
        self.rows[x].update_cell_value(&name, value);

        Ok(())
    }

    // THis should insert row with given column limiters
    /// Insert a row to given number
    ///
    /// This can yield out of rnage error
    pub fn insert_row(&mut self, row_number: usize, source: Option<&Vec<Value>>) -> DcsvResult<()> {
        if row_number > self.get_row_count() {
            return Err(DcsvError::InvalidColumn(format!(
                "Cannot add row to out of range position : {}",
                row_number
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
        self.rows.insert(row_number, new_row);
        Ok(())
    }

    /// Delete a row with given row_number
    ///
    /// This doesn't fail but silent do nothing if number is out of range
    pub fn delete_row(&mut self, row_number: usize) -> Option<Row> {
        let row_count = self.get_row_count();
        if row_count == 0 || row_count < row_number {
            return None;
        }
        Some(self.rows.remove(row_number))
    }

    /// Insert a column with given column information
    pub fn insert_column(
        &mut self,
        column_number: usize,
        column_name: &str,
        column_type: ValueType,
        limiter: Option<ValueLimiter>,
        placeholder: Option<Value>,
    ) -> DcsvResult<()> {
        if column_number > self.get_column_count() {
            return Err(DcsvError::InvalidColumn(format!(
                "Cannot add column to out of range position : {}",
                column_number
            )));
        }
        if let Some(_) = self.try_get_column_index(column_name) {
            return Err(DcsvError::InvalidColumn(format!(
                "Cannot add existing column or number named column = \"{}\"",
                column_name
            )));
        }
        if let Ok(_) = column_name.parse::<isize>() {
            return Err(DcsvError::InvalidColumn(format!(
                "Cannot add number named column"
            )));
        }
        let new_column = Column::new(column_name, column_type, limiter);
        let default_value = new_column.get_default_value();
        for row in &mut self.rows {
            row.insert_cell(
                &new_column.name,
                placeholder.clone().unwrap_or(default_value.clone()),
            );
        }
        self.columns.insert(column_number, new_column);
        Ok(())
    }

    /// Delete a column with given index
    pub fn delete_column(&mut self, column_number: usize) -> DcsvResult<()> {
        let name = self.get_column_if_valid(0, column_number)?.name.to_owned();

        for row in &mut self.rows {
            row.remove_cell(&name);
        }

        self.columns.remove(column_number);

        // If column is empty, drop all rows
        if self.get_column_count() == 0 {
            self.rows = vec![];
        }

        Ok(())
    }

    /// Set a limiter to a column
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
                return Err(DcsvError::InvalidRowData(format!(
                    "Failed to get row data while setting limiter",
                )));
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

    /// Export schema as string form
    pub fn export_schema(&self) -> String {
        let mut schema = format!("{}\n", SCHEMA_HEADER);
        for col in &self.columns {
            let mut line = col.name.clone() + ",";
            let limiter = &col.limiter;
            line.push_str(&limiter.get_type().to_string());
            line.push_str(",");
            line.push_str(
                &limiter
                    .get_default()
                    .map(|s| s.to_string())
                    .unwrap_or(String::new()),
            );
            line.push_str(",");
            line.push_str(
                &limiter
                    .get_variant()
                    .map(|s| s.iter().map(|s| s.to_string()).collect::<Vec<String>>())
                    .unwrap_or(vec![])
                    .join(" "),
            );
            line.push_str(",");
            line.push_str(
                &limiter
                    .get_pattern()
                    .map(|s| s.to_string())
                    .unwrap_or(String::new()),
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
    fn is_valid_cell_coordinate(&self, x: usize, y: usize) -> bool {
        if x < self.get_row_count() {
            if y < self.get_column_count() {
                return true;
            }
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
                return Err(DcsvError::InvalidCellData(format!(
                    "Given cell data failed to match limiter's restriction",
                )));
            }
        } else {
            return Err(DcsvError::InvalidRowData(format!(
                "Given column number \"{}\" doesn't exist",
                column
            )));
        }
    }

    /// Check if given values' length match column's legnth
    fn check_row_length(&self, values: &Vec<Value>) -> DcsvResult<()> {
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

    // </DRY>

    // <EXT>
    pub fn get_row_count(&self) -> usize {
        self.rows.len()
    }

    pub fn get_column_count(&self) -> usize {
        self.columns.len()
    }

    /// Drop all data from self
    pub fn drop(&mut self) {
        self.columns.clear();
        self.rows.clear();
    }

    // </EXT>
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

#[derive(Clone, Debug)]
pub struct Column {
    pub name: String,
    pub column_type: ValueType,
    pub limiter: ValueLimiter,
}

impl Column {
    pub fn new(name: &str, column_type: ValueType, limiter: Option<ValueLimiter>) -> Self {
        Self {
            name: name.to_string(),
            column_type,
            limiter: limiter.unwrap_or(ValueLimiter::default()),
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_column_type(&self) -> &ValueType {
        &self.column_type
    }

    pub fn rename(&mut self, new_name: &str) -> String {
        std::mem::replace(&mut self.name, new_name.to_string())
    }

    pub fn set_limiter(&mut self, limiter: ValueLimiter) {
        self.column_type = limiter.get_type();
        self.limiter = limiter;
    }

    pub fn get_default_value(&self) -> Value {
        // has default
        if let Some(def) = self.limiter.get_default() {
            return def.clone();
        }

        // has variant
        let variant = self.limiter.get_variant();
        if let Some(vec) = variant {
            if vec.len() != 0 {
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

#[derive(Clone, Debug)]
pub struct Row {
    pub values: HashMap<String, Value>,
}

impl Row {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn to_string(&self, columns: &Vec<Column>) -> DcsvResult<String> {
        let mut acc = String::new();
        for col in columns {
            acc.push_str(
                &self
                    .values
                    .get(&col.name)
                    .ok_or(DcsvError::InvalidColumn(
                        "Given column was not present thus cannot construct row string".to_string(),
                    ))?
                    .to_string(),
            );
            acc.push(',');
        }
        acc.pop(); // Remove trailing comma
        Ok(acc)
    }

    pub fn rename_column(&mut self, name: &str, new_name: &str) {
        let previous = self.values.remove(name);

        if let Some(prev) = previous {
            self.values.insert(new_name.to_string(), prev);
        }
    }

    pub fn insert_cell(&mut self, key: &str, value: Value) {
        self.values.insert(key.to_string(), value);
    }

    pub fn get_cell_value(&self, key: &str) -> Option<&Value> {
        self.values.get(key)
    }

    pub fn update_cell_value(&mut self, key: &str, value: Value) {
        if let Some(v) = self.values.get_mut(key) {
            *v = value;
        }
    }

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

    pub fn remove_cell(&mut self, key: &str) {
        self.values.remove(key);
    }
}

/// Read only data
///
/// Columns and rows are all simple string container
#[derive(Debug)]
pub struct ReadOnlyData {
    pub columns: Vec<String>,
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
            columns: data.columns.iter().map(|c| c.name.clone()).collect(),
            rows,
        }
    }
}

#[derive(Debug)]
pub struct ReadOnlyDataRef<'data> {
    pub columns: Vec<&'data str>,
    pub rows: Vec<Vec<&'data Value>>,
}

impl<'data> ReadOnlyDataRef<'data> {
    pub fn to_owned(&self) -> ReadOnlyData {
        ReadOnlyData {
            columns: self.columns.iter().map(|c| c.to_string()).collect(),
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
            columns: data.columns.iter().map(|c| c.name.as_str()).collect(),
            rows,
        }
    }
}
