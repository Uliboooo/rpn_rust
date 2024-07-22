use chrono::Local;
use colored::Colorize;
use get_input::get_input;
use regex::Regex;
use std::env;
use std::fs::OpenOptions;
use std::io::BufRead;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::sync::OnceLock;

static CURRENT_DIR: OnceLock<PathBuf> = OnceLock::new();

enum StatusCode {
    Ok,
    IncludeNonComptableCharacter,
    FormulaNotEntered,
    NoSpaceBetweenOperators,
    ImsufficientOperand,
    CannotConvertToNum,
    OperatorNotEntered,
    UndefinedOperator,
    ResultTooMuch,
    // UnknownError,
}

impl StatusCode {
    fn to_string(&self) -> String {
        format!(
            "{}",
            match self {
                StatusCode::Ok => "",
                StatusCode::IncludeNonComptableCharacter => "計算不可能な文字が含まれています。",
                StatusCode::FormulaNotEntered => "式が入力されていません。",
                StatusCode::NoSpaceBetweenOperators => "演算子間にスペースが含まれていません。",
                StatusCode::ImsufficientOperand => "非演算子(数)が足りない可能性があります。",
                StatusCode::CannotConvertToNum => "数値に変換できませんでした。",
                StatusCode::OperatorNotEntered => "演算子が入力されいない可能性があります。",
                StatusCode::UndefinedOperator => "未定義の演算子が使用されました。",
                StatusCode::ResultTooMuch => "計算結果が大きすぎます。",
                // StatusCode::UnknownError => "原因不明のエラーです。",
            }
            .to_string(),
            // "もう一度計算してください。",
        )
        .to_string()
    }
}

struct History {
    //日付、成否、入力された式、結果もしくはエラーコード
    date: String,
    status: StatusCode,
    formula: String,
    solution: Result<f64, String>,
}

struct SolutionResult {
    solution: Result<f64, String>,
    status_code: StatusCode,
}

fn show_error(error_code_num: &StatusCode) {
    eprintln!(
        "{}\nもう一度入力してください",
        error_code_num.to_string().red()
    );
}

fn status_code_manage(result_status: &StatusCode) {
    match result_status {
        StatusCode::Ok => return,
        _ => show_error(&result_status),
    }
}

fn check_unavailable_character(checked_string: &String) -> bool {
    //入力に演算不可能な文字があった場合false
    let re = Regex::new("[^+\\-*/%1234567890 ]").unwrap();
    if re.is_match(&checked_string) {
        false
    } else {
        true
    }
}

fn check_is_operator(checked_string: &String) -> bool {
    //演算子が存在しない場合
    let re = Regex::new(r"[+\-*/%]").unwrap();
    if re.is_match(&checked_string) {
        true
    } else {
        false
    }
}

fn check_length(checked_string: &String) -> bool {
    //式が入力されていない場合
    if checked_string.len() <= 1 {
        false
    } else {
        true
    }
}

fn check_halfspace(checked_string: &String) -> bool {
    //演算子の間のスペース
    let re = Regex::new(r"\d[^\w\s]").unwrap();
    if re.is_match(&checked_string) {
        false
    } else {
        true
    }
}

fn check_syntax(checked_string: &String) -> Result<bool, StatusCode> {
    //入力された式のチェック
    if check_unavailable_character(checked_string) == false {
        Err(StatusCode::IncludeNonComptableCharacter)
    } else if check_length(checked_string) == false {
        Err(StatusCode::FormulaNotEntered)
    } else if check_halfspace(checked_string) == false {
        Err(StatusCode::NoSpaceBetweenOperators)
    } else if check_is_operator(checked_string) == false {
        Err(StatusCode::OperatorNotEntered)
    } else {
        Ok(true)
    }
}

fn delimit(input: &String) -> Vec<&str> {
    //文字列を空白で区切りベクタにして返す
    input.split_whitespace().collect()
}

fn is_numeric(input: &str) -> bool {
    //入力が数値ならtrue, 演算子ならfalse
    match input.parse::<f64>() {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn to_num(input_str: &str) -> Result<f64, StatusCode> {
    //&strを数値に変換
    match input_str.parse::<f64>() {
        Ok(n) => Ok(n),
        Err(_) => Err(StatusCode::CannotConvertToNum),
    }
}

fn calculation(operand_1: f64, operand_2: f64, operator: &str) -> Result<f64, StatusCode> {
    //演算
    Ok(match operator {
        "+" => operand_1 + operand_2,
        "-" => operand_1 - operand_2,
        "*" => operand_1 * operand_2,
        "/" => operand_1 / operand_2,
        "%" => operand_1 % operand_2,
        "**" => power(operand_1, operand_2),
        _ => return Err(StatusCode::UndefinedOperator),
    })
}

fn power(operand_1: f64, operand_2: f64) -> f64 {
    //指数演算
    let mut power_result = operand_1;
    for _ in 0..operand_2 as i64 - 1 {
        power_result *= operand_1
    }
    power_result
}

fn accuracy_infinite(result_f64: f64) -> Result<(), StatusCode> {
    match result_f64.is_infinite() {
        true => Err(StatusCode::ResultTooMuch),
        false => Ok(()),
    }
}

fn stack_manage(delimited_input: Vec<&str>) -> Result<f64, StatusCode> {
    //stackの制御
    let mut stack = Vec::<f64>::new();
    for i in delimited_input {
        if is_numeric(i) == true {
            //オペランドの場合
            stack.push(match to_num(i) {
                Ok(result) => result,
                Err(error_code) => return Err(error_code),
            });
        } else {
            //演算子の場合
            if stack.len() < 2 {
                return Err(StatusCode::ImsufficientOperand);
            }; //引数不足
            let result = match calculation(stack[stack.len() - 2], stack[stack.len() - 1], i) {
                Ok(result_) => result_,
                Err(error_code) => return Err(error_code),
            };
            for _ in 0..2 {
                stack.remove(stack.len() - 1); //stackのクリア
            }
            stack.push(result); //結果の挿入
        }
    }
    match accuracy_infinite(stack[stack.len() - 1]) {
        Ok(_) => {}
        Err(error_code) => return Err(error_code),
    }
    if stack.len() > 1 {
        return Err(StatusCode::ImsufficientOperand);
    } else {
        return Ok(stack[stack.len() - 1]);
    }
}

fn status_code_to_boolstring(status_code: &StatusCode) -> String {
    match status_code {
        StatusCode::Ok => "成功".to_string(),
        _ => "失敗".to_string(),
    }
}

fn history_to_string(content: History) -> String {
    //書き込み可能なStringに変換
    let log_content = format!(
        "{},{},{},{},{}\n",
        content.date.to_string(),
        status_code_to_boolstring(&content.status),
        content.formula,
        match content.solution {
            Ok(solution) => solution.to_string(),
            Err(error_msg) => error_msg,
        },
        content.status.to_string()
    );
    log_content
}

fn add_data_csv(path: &Path, content: History) {
    //データをcsvに追加
    let file = match OpenOptions::new().write(true).append(true).open(path) {
        Ok(file) => file,
        Err(_) => return,
    };
    let mut bw = BufWriter::new(file);
    match bw.write(history_to_string(content).as_bytes()) {
        Ok(_) => match bw.flush() {
            Ok(_) => {}
            Err(_) => return,
        },
        Err(_) => return,
    }
}

fn add_column_csv(path: &Path) -> Result<(), std::io::Error> {
    //カラムを追加
    let file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(path)?;
    let reader = BufReader::new(file);
    let column = "日付,成否,入力された式,結果,ステータスコード";
    let mut lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;
    if lines.is_empty() || lines[0] != column {
        lines.insert(0, column.to_string());
        let mut file = OpenOptions::new().write(true).truncate(true).open(path)?;
        for line in lines {
            writeln!(file, "{}", line)?;
        }
    }
    Ok(())
}

fn log_history(log_content: History) {
    let path = &CURRENT_DIR
        .get()
        .expect("failed get dir")
        .join("./history.csv");
    match add_column_csv(path) {
        Ok(_) => (),
        Err(_) => return,
    };
    add_data_csv(path, log_content);
}

fn main() {
    let path = env::current_exe().expect("Failed get path");
    let dir = path.parent().expect("Failed get dir").to_path_buf();
    match CURRENT_DIR.set(dir.clone()) {
        Ok(_) => {}
        Err(_) => {}
    };
    println!("{:?}", CURRENT_DIR.get());
    loop {
        println!("式を入力してください。\"n\"で終了\n例: 1 2 + 3 4 + +(値や演算子は半角スペースで区切ってください。)\n使用可能演算子: 加(+)減(-)乗(*)除(/)余(%)指(**)");
        let input_formula = get_input();
        if &input_formula == &"n".to_string() {
            break;
        };
        let result = match check_syntax(&input_formula) {
            Ok(_) => {
                //check_syntaxが通った
                let delimited_input_fomula = delimit(&input_formula);
                match stack_manage(delimited_input_fomula) {
                    Ok(result) => SolutionResult {
                        solution: Ok(result),
                        status_code: (StatusCode::Ok),
                    },
                    Err(error_code) => SolutionResult {
                        solution: Err("error".to_string()),
                        status_code: (error_code),
                    },
                }
            }
            Err(error_code) => {
                //check_syntaxが通らなかった場合
                SolutionResult {
                    solution: Err("error".to_string()),
                    status_code: (error_code),
                }
            }
        };
        status_code_manage(&result.status_code);
        match result.solution {
            //結果がある場合のみ表示
            Ok(solution) => println!("{}\n", solution),
            Err(_) => println!(""),
        };
        log_history(History {
            date: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            status: result.status_code,
            formula: input_formula.clone(),
            solution: match result.solution {
                Ok(result) => Ok(result),
                Err(errror_msg) => Err(errror_msg),
            },
        });
    }
}
