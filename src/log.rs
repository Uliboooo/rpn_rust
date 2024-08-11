use crate::{ErrorCode, Solution, CURRENT_DIR};
use chrono::{DateTime, Local};
use core::fmt;
use std::{
    fs::OpenOptions,
    io::{BufRead, BufReader, BufWriter, Write},
    path::PathBuf,
};

pub struct History {
    pub date: DateTime<Local>,
    pub success_or_failed: SuccessOrFailed,
    pub formula: String,
    pub solution: Solution,
}

impl History {
    fn to_line(&self) -> String {
        format!(
            "{},{},{},{}\n",
            &self.date.format("%Y-%m-%d %H:%M:%S"),
            &self.success_or_failed,
            &self.formula,
            match &self.solution {
                Solution::Success(ans) => ans.to_string(),
                Solution::Failed(error) => error.to_string(),
            },
        )
    }
}

pub enum SuccessOrFailed {
    Success,
    Failed,
}

impl fmt::Display for SuccessOrFailed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SuccessOrFailed::Success => write!(f, "success"),
            SuccessOrFailed::Failed => write!(f, "failed"),
        }
    }
}

fn add_csv_line(csv_path: &PathBuf, content: &String) -> Result<(), ErrorCode> {
    let file = match OpenOptions::new().append(true).open(csv_path) {
        Ok(file) => file,
        Err(_) => return Err(ErrorCode::FailedAddCsvData),
    };
    let mut bw = BufWriter::new(file);
    match bw.write_all(content.as_bytes()) {
        Ok(_) => match bw.flush() {
            Ok(_) => Ok(()),
            Err(_) => Err(ErrorCode::FailedAddCsvData),
        },
        Err(_) => Err(ErrorCode::FailedAddCsvData),
    }
}

fn add_csv_column(csv_path: PathBuf) -> Result<(), ErrorCode> {
    let file = match OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(&csv_path)
    {
        Ok(file) => file,
        Err(_) => return Err(ErrorCode::FailedAddCsvColumn),
    };
    let column_str = "日付,成否,式,結果\n".to_string();
    if let Some(line) = BufReader::new(file).lines().next() {
        match line {
            Ok(line) => {
                if line.trim() == column_str.trim() {
                    return Ok(());
                }
            }
            Err(_) => return Err(ErrorCode::FailedAddCsvColumn),
        }
    }
    match add_csv_line(&csv_path, &column_str) {
        Ok(_) => Ok(()),
        Err(error_code) => Err(error_code),
    }
}

fn to_csv_path() -> PathBuf {
    CURRENT_DIR
        .get()
        .expect("failed get current dir.")
        .join("history.csv")
}

pub fn log_history(log_content: History) -> Result<(), ErrorCode> {
    match add_csv_column(to_csv_path()) {
        Ok(_) => match add_csv_line(&to_csv_path(), &log_content.to_line()) {
            Ok(_) => Ok(()),
            Err(error_code) => Err(error_code),
        },
        Err(error_code) => Err(error_code),
    }
}
