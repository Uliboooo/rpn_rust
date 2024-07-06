use regex::Regex;
use get_input::get_input;
use colored::Colorize;
use std::fs::OpenOptions;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Write;
use chrono::Local;

struct History { //日付、成否、入力された式、結果もしくはエラーコード
    date: String,
    is_success: bool,
    forumula: String,
    solution: f64,
}

fn show_error(error_code_num: u16) { //エラーコードから適切なエラーを表示
    eprintln!(
        "{}{} {}",
        error_code_num.to_string().red(),
        ":".red(),
        match error_code_num {
            0101 => "計算不可能な文字が含まれています。",
            0102 => "式が入力されていない可能性があります。",
            0103 => "演算子の間にスペースが含まれていない可能性があります。",
            0104 => "被演算子(数)が足りない可能性があります。",
            0105 => "数値に変換できませんでした。",
            0106 => "演算子が入力されていない可能性があります。",
            0201 => "未定義の演算子が入力されました。",
            _ => "原因不明のエラーです。",
        }.to_string().red(),
    );
    println!("もう一度入力してください。\n");
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

fn check_syntax(checked_string: &String) -> Result<bool, u16> { //入力された式のチェック
    if check_unavailable_character(checked_string) == false {
        Err(0101)
    } else if check_length(checked_string) == false {
        Err(0102)
    } else if check_halfspace(checked_string) == false {
        Err(0103)
    } else if check_is_operator(checked_string) == false {
        Err(0106)
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

fn to_num(input_str: &str) -> Result<f64, u16> { //&strを数値に変換
    match input_str.parse::<f64>() {
        Ok(n) => Ok(n),
        Err(_) => Err(0105),
    }
}

fn calculation(operand_1: f64, operand_2: f64, operator: &str) -> Result<f64, u16> { //演算
    match operator {
        "+" => Ok(operand_1 + operand_2),
        "-" => Ok(operand_1 - operand_2),
        "*" => Ok(operand_1 * operand_2),
        "/" => Ok(operand_1 / operand_2),
        "%" => Ok(operand_1 % operand_2),
        "**" => Ok(power(operand_1, operand_2)),
        _ => Err(0201),
    }
}

fn power(operand_1: f64, operand_2: f64) -> f64 { //指数演算
    let mut power_result = operand_1;
    for _ in 0..operand_2 as i64 - 1 {
        power_result *= operand_1
    }
    power_result
}

fn stack_manage(delimited_input: Vec<&str>) -> Result<f64, u16>{ //stackの制御
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
            if stack.len() < 2 {return Err(0104)}; //引数不足
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
    if stack.len() > 1 {
        Err(0104)
    } else {
        Ok(stack[stack.len() - 1])
    }
}

fn bool_to_string(is_success: bool) -> String { //trueなら"成功"、falseなら"失敗"を返す
    if is_success == true {"成功".to_string()} else {"失敗".to_string()}
}

fn to_writable_log(history_content: History) -> String { //個別の履歴情報を単一の書き込み可能なStringに変換
    let mut log_content = history_content.date.to_string();
    log_content.push_str(",");
    log_content.push_str(bool_to_string(history_content.is_success).as_str());
    log_content.push_str(",");
    log_content.push_str(history_content.forumula.as_str());
    log_content.push_str(",");
    let solution_str = if history_content.is_success == true {history_content.solution.to_string()} else {format!("エラー: {}", history_content.solution)};
    log_content.push_str(solution_str.as_str());
    log_content.push_str("\n");
    log_content
}

fn add_csv(input_content: String, path: &str) -> Result<(),()>{ //pathに存在するcsvにinputを書き込み
    let file = match OpenOptions::new().append(true).write(true).create(true).open(path) {
        Ok(file) => file,
        Err(_) => {
            return Err(());
        },
    };
    let mut bw = BufWriter::new(file);
    match bw.write(input_content.as_bytes()) {
        Ok(_) => {},
        Err(_) => return Err(()),
    }
    match bw.flush() {
        Ok(_) => return Ok(()),
        Err(_) => return Err(()),
    }
}

fn add_column_csv(path: &str) -> Result<(), std::io::Error> { //カラムの挿入
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let column = "日付,成否,入力された式,結果(or エラーコード)";
    
    let mut lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;
    
    if lines.is_empty() || lines[0] != column {
        lines.insert(0, column.to_string());
        
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(path)?;
        
        for line in lines {
            writeln!(file, "{}", line)?;
        }
    }
    Ok(())
}

fn log_history(input_is_success: bool, input_formula: &String, input_solution: f64) { //履歴を作成、記録
    // エラーが発生したら記録せずスキップ
    let log = History {
        date: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        is_success: input_is_success,
        forumula: input_formula.clone(),
        solution: input_solution,
    };
    let path = "history.csv";
    match add_column_csv(&path){
        Ok(_) => {},
        Err(_) => return,
    };
    match add_csv(to_writable_log(log), &path) { //単一のStringにした履歴を追記
        Ok(_) => {},
        Err(_) => return,
    }
}

fn main() {
    loop {
        println!("式を入力してください。\"n\"で終了\n例: 1 2 + 3 4 + +(値や演算子は半角スペースで区切ってください。)\n使用可能演算子: 加(+)減(-)乗(*)除(/)余(%)指(**)");
        let input_formula = get_input();
        if &input_formula == &"n".to_string() {break;};
        match check_syntax(&input_formula) {
            Ok(_) => {},
            Err(error_code) => {
                show_error(error_code);
                log_history(false, &input_formula, error_code as f64);
                continue;
            }
        }
        let delimited_input_fomula = delimit(&input_formula);
        let result = match stack_manage(delimited_input_fomula) {
            Ok(result_) => {
                log_history(true, &input_formula, result_);
                result_
            }
            Err(error_code) => {
                show_error(error_code);
                log_history(false, &input_formula, error_code as f64);
                continue;
            }
        };
        println!("{}\n", result);
    }
}