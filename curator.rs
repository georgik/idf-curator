#[macro_use]
extern crate json;

use std::env;
use std::fs;
use std::path::Path;

use md5;

fn help() {
    println!("Help");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut property_name = "";
    let mut idf_id = format!("{}","");
    let idf_tools_path_env = "IDF_TOOLS_PATH";

    let idf_tools_path = env::var(idf_tools_path_env).unwrap_or_else(|e| {
        panic!("could not find {}: {}", idf_tools_path_env, e)
    });

    let content = fs::read_to_string(idf_tools_path + "/esp_idf.json")
    .expect("Failure");
    let parsed2 = json::parse(&content.to_string()).unwrap();

    match args.len() {
        // no arguments passed
        1 => {
            println!("My name is 'match_args'. Try passing some arguments!");
        },
        // one argument passed
        3 => {
            let cmd = &args[1];
            property_name = &args[2];
            match &cmd[..] {
                "get-property" => {
                    let git_path = &parsed2[property_name].to_string();
                    let path = Path::new(&git_path);
                    let parent = path.parent().unwrap().to_str();
                    print!("{}", parent.unwrap());
                },
                _ => {
                    eprintln!("error: invalid command");
                    help();
                },
            }
        },
        5 => {
            let cmd = &args[1];
            property_name = &args[2];
            let option_name = &args[3];
            let idf_path = &args[4].replace("\\","/");
            let idf_path_with_slash = format!("{}/", idf_path);
            let digest = md5::compute(idf_path_with_slash);
            idf_id = format!("esp-idf-{:x}", digest);
            let property_path = &parsed2["idfInstalled"][idf_id][property_name].to_string();
            let path = Path::new(&property_path);
            let parent = path.parent().unwrap().to_str();
            print!("{}",  parent.unwrap());
        },
        _ => {
            // show a help message
            help();
        }
    }
}
