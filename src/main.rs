use get_input::get_input;
use regex::Regex;
use std::{env, fmt, path::PathBuf, sync::OnceLock};
use log_history::{self as his};
use chrono::Local;

static CURRENT_DIR: OnceLock<PathBuf> = OnceLock::new();

mod log_history {
    use core::fmt;
    use std::{fs::OpenOptions, io::{BufRead, BufReader, BufWriter, Write}, path::PathBuf};
    use chrono::{DateTime, Local};
    use crate::{ErrorCode, Solution, CURRENT_DIR};

    pub struct History {
        pub date: DateTime<Local>,
        pub success_or_failed: SuccessOrFailed,
        pub formula: String,
        pub solution: Solution,
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

    fn to_csv_line(input: History) -> String {
        format!(
            "{},{},{},{}\n",
            input.date.format("%Y-%m-%d %H:%M:%S"),
            input.success_or_failed,
            input.formula,
            match input.solution {
                Solution::Success(ans) => ans.to_string(),
                Solution::Failed(error) => error.to_string(),
            },
        )
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
        let file = match OpenOptions::new().create(true).truncate(false).read(true).write(true).open(&csv_path) {
            Ok(file) => file,
            Err(_) => return Err(ErrorCode::FailedAddCsvColumn),
        };
        let column_str = "日付,成否,式,結果\n".to_string();
        if let Some(line) = BufReader::new(file).lines().next() {match line {
            Ok(line) => {
                if line.trim() == column_str.trim() {
                    return Ok(())
                }
            },
            Err(_) => return Err(ErrorCode::FailedAddCsvColumn),
        }}
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

    pub fn log_history(log_content: History) -> Result<(), ErrorCode>{
        let csv_line_content = to_csv_line(log_content);
        match add_csv_column(to_csv_path()){
            Ok(_) => {
                match add_csv_line(&to_csv_path(), &csv_line_content) {
                    Ok(_) => Ok(()),
                    Err(error_code) => Err(error_code),
                }
            },
            Err(error_code) => Err(error_code),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Solution {
    Success(f64),
    Failed(ErrorCode),
}

#[derive(Debug, Clone, PartialEq)]
enum ErrorCode {
    NonComptableCharacter,
    FormulaNotEntered,
    NoSpaceBetweenOperators,
    OperatorNotEntered,
    FailedConvertNum,
    ImsufficientOperand,
    NotComplete,
    UndefinedOperator,
    ResultTooMuch,
    FailedAddCsvColumn,
    FailedAddCsvData,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorCode::NonComptableCharacter => write!(f, "計算不能な文字が含まれています。"),
            ErrorCode::FormulaNotEntered => write!(f, "式が入力されていない可能性があります。"),
            ErrorCode::NoSpaceBetweenOperators => write!(f, "演算子間にスペースが含まれていません。"),
            ErrorCode::OperatorNotEntered => write!(f, "演算子が入力されていません。"),
            ErrorCode::FailedConvertNum => write!(f, "数値に変換できませんでした。"),
            ErrorCode::ImsufficientOperand => write!(f, "被演算子(数値)が不足しています。"),
            ErrorCode::NotComplete => write!(f, "計算が正常に完了しませんでした。"),
            ErrorCode::UndefinedOperator => write!(f, "未定義演算子が使用されています。"),
            ErrorCode::ResultTooMuch => write!(f, "計算結果が大きすぎます。"),
            ErrorCode::FailedAddCsvColumn => write!(f, "ログファイル(csv)へカラムを追加できませんでした。"),
            ErrorCode::FailedAddCsvData => write!(f, "ログファイル(csv)へデータを追加できませんでした。")
        }
    }
}

fn check_unavailable_character(checked_string: &str) -> bool {
    let re = Regex::new("[^+\\-*/%1234567890 ]").unwrap();
    //reは正常値以外
    //reにマッチした場合は不正値が含まれるため、falseを返す
    !re.is_match(checked_string)
}

fn check_length(checked_string: &str) -> bool {
    //式が入力されていない場合
    checked_string.len() > 1
}

fn check_halfspace(checked_string: &str) -> bool {
    //演算子の間のスペース
    let re = Regex::new(r"\d[^\w\s]").unwrap();
    !re.is_match(checked_string)
}

fn check_is_operator(checked_string: &str) -> bool {
    //演算子が存在しない場合
    let re = Regex::new(r"[+\-*/%]").unwrap();
    re.is_match(checked_string)
}

fn check_syntax(checked_string: &str) -> Result<bool, ErrorCode> {
    //入力された式のチェック
    if !check_unavailable_character(checked_string) {
        Err(ErrorCode::NonComptableCharacter)
    } else if !check_length(checked_string) {
        Err(ErrorCode::FormulaNotEntered)
    } else if !check_halfspace(checked_string) {
        Err(ErrorCode::NoSpaceBetweenOperators)
    } else if !check_is_operator(checked_string) {
        Err(ErrorCode::OperatorNotEntered)
    } else {
        Ok(true)
    }
}

fn to_vec(formula_str: &str) -> Vec<&str> {
    formula_str.split_whitespace().collect()
}

fn to_num(input: &str) -> Result<f64, ErrorCode> {
    match input.parse::<f64>() {
        Ok(num) => Ok(num),
        Err(_) => Err(ErrorCode::FailedConvertNum),
    }
}

fn calculation(operands: (f64, f64), operator: &str) -> Result<f64, ErrorCode> {
    match operator {
        "+" => Ok(operands.0 + operands.1),
        "-" => Ok(operands.0 - operands.1),
        "*" => Ok(operands.0 * operands.1),
        "/" => Ok(operands.0 / operands.1),
        "%" => Ok(operands.0 % operands.1),
        "^" => Ok(operands.0.powf(operands.1)),
        _ => Err(ErrorCode::UndefinedOperator),
    }
}

fn manage_calculate(formula_vec: Vec<&str>) -> Result<f64, ErrorCode> {
    let mut operands = Vec::<f64>::new();
    for i in formula_vec {
        match to_num(i) {
            Ok(num) => {
                //数値の場合
                operands.push(num)
            }
            Err(_) => {
                //演算子
                if operands.len() < 2 {
                    return Err(ErrorCode::ImsufficientOperand);
                }
                let result = match calculation(
                    (operands[operands.len() - 2], operands[operands.len() - 1]),
                    i,
                ) {
                    Ok(ans) => ans,
                    Err(error_code) => return Err(error_code),
                };
                operands.drain(operands.len() - 2..operands.len());
                operands.push(result);
            }
        }
    }
    if operands.len() > 1 {
        Err(ErrorCode::NotComplete)
    } else if operands[operands.len() - 1].is_infinite() {
        Err(ErrorCode::ResultTooMuch)
    } else {
        Ok(operands[operands.len() - 1])
    }
}

fn judge_success_failed(con: Solution) -> log_history::SuccessOrFailed{
    match con {
        Solution::Success(_) => log_history::SuccessOrFailed::Success,
        Solution::Failed(_) => log_history::SuccessOrFailed::Failed,
    }
}

fn main() {
    if CURRENT_DIR.set(env::current_exe().expect("failed get current_exe.").parent().expect("failed get parent.").to_path_buf()).is_ok() {};
    loop {
        println!("式を入力してください。\"n\"で終了。\n例: (1 + 2)x(3 + 4) ---> 1 2 + 3 4 + *(半角スペースで区切ってください)\n演算子: 加(+)減(-)乗(*)除(/)余(%)指(^)");
        let input_formula_str = get_input();
        if input_formula_str == "n" {
            break;
        }
        let result: Solution = match check_syntax(&input_formula_str) {
            Ok(_) => {
                let formula_vec = to_vec(&input_formula_str);
                match manage_calculate(formula_vec) {
                    Ok(ans) => Solution::Success(ans),
                    Err(error_code) => Solution::Failed(error_code),
                }
            }
            Err(error_code) => Solution::Failed(error_code),
        };
        match &result {
            Solution::Success(ans) => println!("Ans: {}\n", ans),
            Solution::Failed(error_code) => eprintln!("Error: {}\nもう一度入力してください。\n", error_code),
        }
        match his::log_history(his::History{
            date: Local::now(),
            success_or_failed: judge_success_failed(result.clone()),
            formula: input_formula_str,
            solution: result,
        }){
            Ok(_) => {},
            Err(error_code) => eprintln!("Error: {}", error_code),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{check_syntax, manage_calculate, to_vec, ErrorCode, Solution};
    #[test]
    fn it_workd() {
        let formula_vec = vec![
            ("1 2 +", Solution::Success(3.0)),
            ("1 2 + 3 4 + +", Solution::Success(10.0)),
            ("a", Solution::Failed(ErrorCode::NonComptableCharacter)),
            ("", Solution::Failed(ErrorCode::FormulaNotEntered)),
            ("1+", Solution::Failed(ErrorCode::NoSpaceBetweenOperators)),
            ("1 +", Solution::Failed(ErrorCode::ImsufficientOperand)),
            ("1 2 + 3 4 +", Solution::Failed(ErrorCode::NotComplete)),
        ];
        for input_formula_str in formula_vec {
            let result: Solution = match check_syntax(input_formula_str.0) {
                Ok(_) => {
                    let formula_vec = to_vec(input_formula_str.0);
                    match manage_calculate(formula_vec) {
                        Ok(ans) => Solution::Success(ans),
                        Err(error_code) => Solution::Failed(error_code),
                    }
                }
                Err(error_code) => Solution::Failed(error_code),
            };
            assert_eq!(result, input_formula_str.1);
        }
    }
}
