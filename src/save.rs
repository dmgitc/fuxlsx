use crate::workbook::{CellCoordinate, CellValue, Changeset};
use anyhow::{Context, Result};
use calamine::{Data, Reader, open_workbook_auto};
use rust_xlsxwriter::{Format, Workbook as XlsxWorkbook, Worksheet};
use std::fs;
use std::path::Path;

/// Save the workbook with changes applied from the changeset.
/// Creates a backup of the original file before overwriting.
pub fn save_workbook_with_changes(
    original_path: &Path,
    changeset: &Changeset,
) -> Result<()> {
    // Open the original workbook for reading
    let mut workbook = open_workbook_auto(original_path)
        .with_context(|| format!("Failed to open workbook for reading: {}", original_path.display()))?;

    // Create a new workbook for writing
    let mut output_workbook = XlsxWorkbook::new();

    // Get all sheet names
    let sheet_names = workbook.sheet_names();

    // Copy each sheet to the new workbook, applying edits
    for sheet_name in sheet_names {
        // Get the range for this sheet
        let range = workbook
            .worksheet_range(&sheet_name)
            .with_context(|| format!("Failed to read sheet: {}", sheet_name))?;

        // Create a new worksheet
        let worksheet = output_workbook.add_worksheet();
        worksheet.set_name(&sheet_name)
            .with_context(|| format!("Failed to set worksheet name: {}", sheet_name))?;

        // Get edits for this sheet
        let edits_for_sheet = changeset.edits_for_sheet(&sheet_name);

        // Copy cells from the original sheet, applying edits
        for (row_idx, row) in range.rows().enumerate() {
            for (col_idx, cell) in row.iter().enumerate() {
                // Check if this cell has an edit
                let coord = CellCoordinate {
                    sheet_name: sheet_name.clone(),
                    row: row_idx,
                    col: col_idx,
                };

                let has_edit = edits_for_sheet.iter().any(|(c, _)| **c == coord);

                if has_edit {
                    // Write the edited value
                    if let Some((_, edit)) = edits_for_sheet.iter().find(|(c, _)| **c == coord) {
                        write_cell_value(worksheet, row_idx as u32, col_idx as u16, &edit.new_value)?;
                    }
                } else {
                    // Write the original value
                    write_calamine_data(worksheet, row_idx as u32, col_idx as u16, cell)?;
                }
            }
        }
    }

    // Create backup of original file
    let backup_path = original_path.with_extension("xlsx.bak");
    if original_path.exists() {
        fs::copy(original_path, &backup_path)
            .with_context(|| format!("Failed to create backup at: {}", backup_path.display()))?;
    }

    // Write to a temp file first
    let temp_path = original_path.with_extension("xlsx.tmp");
    output_workbook.save(&temp_path)
        .with_context(|| format!("Failed to save workbook to temp file: {}", temp_path.display()))?;

    // Rename temp file to original
    fs::rename(&temp_path, original_path)
        .with_context(|| format!("Failed to rename temp file to: {}", original_path.display()))?;

    // Remove backup on success
    let _ = fs::remove_file(&backup_path);

    Ok(())
}

/// Write a CellValue to a worksheet cell
fn write_cell_value(
    worksheet: &mut Worksheet,
    row: u32,
    col: u16,
    value: &CellValue,
) -> Result<()> {
    match value {
        CellValue::Empty => {
            // Write empty string for empty cells
            worksheet.write_string(row, col, "")?;
        }
        CellValue::String(s) => {
            worksheet.write_string(row, col, s)?;
        }
        CellValue::Int(n) => {
            worksheet.write_number(row, col, *n as f64)?;
        }
        CellValue::Float(n) => {
            worksheet.write_number(row, col, *n)?;
        }
        CellValue::Bool(b) => {
            worksheet.write_boolean(row, col, *b)?;
        }
        CellValue::Error(e) => {
            // Write error as string
            worksheet.write_string(row, col, e)?;
        }
        CellValue::DateTime(serial) => {
            // Write as Excel date/time with number format
            let format = Format::new().set_num_format("yyyy-mm-dd hh:mm:ss");
            worksheet.write_number_with_format(row, col, *serial, &format)?;
        }
    }
    Ok(())
}

/// Write calamine Data to a worksheet cell
fn write_calamine_data(
    worksheet: &mut Worksheet,
    row: u32,
    col: u16,
    data: &Data,
) -> Result<()> {
    match data {
        Data::Empty => {
            // Don't write anything for empty cells
        }
        Data::String(s) => {
            worksheet.write_string(row, col, s)?;
        }
        Data::Int(n) => {
            worksheet.write_number(row, col, *n as f64)?;
        }
        Data::Float(n) => {
            worksheet.write_number(row, col, *n)?;
        }
        Data::Bool(b) => {
            worksheet.write_boolean(row, col, *b)?;
        }
        Data::Error(e) => {
            // Write error reference as string
            worksheet.write_string(row, col, &format!("{:?}", e))?;
        }
        Data::DateTime(dt) => {
            // Write as Excel date/time
            let format = Format::new().set_num_format("yyyy-mm-dd hh:mm:ss");
            worksheet.write_number_with_format(row, col, dt.as_f64(), &format)?;
        }
        Data::DateTimeIso(s) => {
            // Write ISO date string
            worksheet.write_string(row, col, s)?;
        }
        Data::DurationIso(s) => {
            // Write duration as string
            worksheet.write_string(row, col, s)?;
        }
    }
    Ok(())
}
