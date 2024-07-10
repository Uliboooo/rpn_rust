use regex::Regex;
use get_input::get_input;
use colored::Colorize;
use std::env;
use std::fs::OpenOptions;
use std::io::BufRead;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;
use chrono::Local;

struct History { //日付、成否、入力された式、結果もしくはエラーコード
    date: String,
    status: (u8, u8),
    forumula: String,
    solution: Result<f64, String>,
}

struct SolutionResult {
    solution: Result<f64, String>,
    status: (u8, u8),
}

fn join_error_code(u8_code: (u8, u8)) -> u16{
    u8_code.0 as u16 * 100 + u8_code.1 as u16
}

fn show_error(error_code_num: (u8, u8)) { //エラーコードから適切なエラーを表示
    eprintln!(
        "{}{} {}",
        join_error_code(error_code_num).to_string().red(),
        ":".red(),
        match error_code_num.0 {
            01 => match error_code_num.1 {
                01 => "計算不可能な文字が含まれています。",
                02 => "式が入力されていない可能性があります。",
                03 => "演算子の間にスペースが含まれていない可能性があります。",
                04 => "被演算子(数)が足りない可能性があります。",
                05 => "数値に変換できませんでした。",
                06 => "演算子が入力されていない可能性があります。",
                _ => "原因不明のエラーです。"
            },
            02 => match error_code_num.1 {
                01 => "未定義の演算子が入力されました。",
                02 => "計算結果が大きすぎます。",
                _ => "原因不明のエラーです。",
            }
            _ => "原因不明のエラーです。",
        }.to_string().red()
    );
    println!("もう一度入力してください。\n");
}

fn status_code_manage(result_status: (u8, u8)) {
    match result_status.0 {
        00 => return,
        _ => show_error(result_status),
    }
}

fn check_unavailable_character(checked_string: &String) -> bool { //入力に演算不可能な文字があった場合false
    let re = Regex::new("[^+\\-*/%1234567890 ]").unwrap();
    if re.is_match(&checked_string) {
        false
    } else {
        true
    }
}

fn check_is_operator (checked_string: &String) -> bool { //演算子が存在しない場合
    let re = Regex::new(r"[+\-*/%]").unwrap();
    if re.is_match(&checked_string){
        true
    } else {
        false
    }
}

fn check_length(checked_string: &String) -> bool{ //式が入力されていない場合
    if checked_string.len() <= 1 {
        false
    } else {
        true
    }
}

fn check_halfspace(checked_string: &String) -> bool { //演算子の間のスペース
    let re = Regex::new(r"\d[^\w\s]").unwrap();
    if re.is_match(&checked_string) {
        false
    } else {
        true
    }
}

fn check_syntax(checked_string: &String) -> Result<bool, (u8, u8)> { //入力された式のチェック
    if check_unavailable_character(checked_string) == false {
        Err((01,01))
    } else if check_length(checked_string) == false {
        Err((01, 02))
    } else if check_halfspace(checked_string) == false {
        Err((01, 03))
    } else if check_is_operator(checked_string) == false {
        Err((01, 06))
    } else {
        Ok(true)
    }
}

fn delimit(input: &String) -> Vec<&str>{ //文字列を空白で区切りベクタにして返す
    input.split_whitespace().collect()
}

fn is_numeric(input: &str) -> bool { //入力が数値ならtrue, 演算子ならfalse
    match input.parse::<f64>() {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn to_num(input_str: &str) -> Result<f64, (u8, u8)> { //&strを数値に変換
    match input_str.parse::<f64>() {
        Ok(n) => Ok(n),
        Err(_) => Err((01,05)),
    }
}

fn calculation(operand_1: f64, operand_2: f64, operator: &str) -> Result<f64, (u8, u8)> { //演算
    Ok(
        match operator {
        "+" => operand_1 + operand_2,
        "-" => operand_1 - operand_2,
        "*" => operand_1 * operand_2,
        "/" => operand_1 / operand_2,
        "%" => operand_1 % operand_2,
        "**" => power(operand_1, operand_2),
        _ => return Err((02, 01)),
        }
    )
}

fn power(operand_1: f64, operand_2: f64) -> f64 { //指数演算
    let mut power_result = operand_1;
    for _ in 0..operand_2 as i64 - 1 {
        power_result *= operand_1
    }
    power_result
}

fn accuracy_infinite(result_f64: f64) -> Result<(), (u8, u8)>{
    match result_f64.is_infinite() {
        true => Err((02, 02)),
        false => Ok(()),
    }
}

fn stack_manage(delimited_input: Vec<&str>) -> Result<f64, (u8, u8)>{ //stackの制御
    let mut stack = Vec::<f64>::new();
    for i in delimited_input {
        if is_numeric(i) == true { //オペランドの場合
            stack.push(
                match to_num(i) {
                    Ok(result) => result,
                    Err(error_code) => return Err(error_code),
                }
            );
        } else { //演算子の場合
            if stack.len() < 2 {return Err((01, 04))}; //引数不足
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
    match accuracy_infinite(stack[stack.len() -1]) {
        Ok(_) => {},
        Err(error_code) => return Err(error_code)
    }
    if stack.len() > 1 {
        return Err((01, 04))
    } else {
        return Ok(stack[stack.len() - 1])
    }
}

fn status_code_to_boolstring(status_code: (u8, u8)) -> String {
    match status_code.0 {
        00 => "成功".to_string(),
        _ => "失敗".to_string(),
    }
}

fn u8_code_to_string(code: (u8, u8)) -> String {
    join_error_code(code).to_string()
}

fn history_to_string(content: History) -> String { //書き込み可能なStringに変換
    let log_content = format!(
            "{},{},{},{},{}\n",
            content.date.to_string(),
            status_code_to_boolstring(content.status),
            content.forumula, 
            match content.solution{
                Ok(solution) => solution.to_string(),
                Err(error_msg) => error_msg,
            }, 
            u8_code_to_string(content.status)
        );
    log_content
}

fn add_data_csv(path: &Path, content: History) { //データをcsvに追加
    let file = match OpenOptions::new().write(true).append(true).open(path) {
        Ok(file) => file,
        Err(_) => return,
    };
    let mut bw = BufWriter::new(file);
    match bw.write(history_to_string(content).as_bytes()) {
        Ok(_) => {
            match bw.flush(){
                Ok(_) => {},
                Err(_) => return,
            }
        },
        Err(_) => return,
    }
}

fn add_column_csv(path: &Path) -> Result<(), std::io::Error> { //カラムを追加
    let file = OpenOptions::new().create(true).read(true).write(true).open(path)?;
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
    let path = Path::new("./history.csv");
    match add_column_csv(path) {
        Ok(_) => {()},
        Err(_) => return,
    };
    add_data_csv(path, log_content);
}

fn main() {
    let exe_path = env::current_exe().expect("Failed to get current path");
    let exe_dir = exe_path.parent().expect("Failed to get parent path");
    env::set_current_dir(exe_dir).expect("Failed to set dir");
    loop {
        println!("式を入力してください。\"n\"で終了\n例: 1 2 + 3 4 + +(値や演算子は半角スペースで区切ってください。)\n使用可能演算子: 加(+)減(-)乗(*)除(/)余(%)指(**)");
        let input_formula = get_input();
        if &input_formula == &"n".to_string() {break;};
        let result = match check_syntax(&input_formula) {
            Ok(_) => { //check_syntaxが通った
                let delimited_input_fomula = delimit(&input_formula);
                match stack_manage(delimited_input_fomula) {
                    Ok(result) => {
                        SolutionResult {
                            solution: Ok(result),
                            status: (00, 01),
                        }
                    },
                    Err(error_code) => {
                        SolutionResult {
                            solution: Err("error".to_string()),
                            status: (error_code),
                        }
                    },
                }
            },
            Err(error_code) => { //check_syntaxが通らなかった場合
                SolutionResult {
                    solution: Err("error".to_string()),
                    status: (error_code)
                }
            },
        };
        status_code_manage(result.status);
        match result.solution { //結果がある場合のみ表示
            Ok(solution) => println!("{}", solution),
            Err(_) => {},
        };
        log_history(
            History {
                date: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                status: result.status,
                forumula: input_formula.clone(),
                solution: match result.solution {
                    Ok(result) => Ok(result),
                    Err(errror_msg) => Err(errror_msg),
                }
            }
        );
    }
}