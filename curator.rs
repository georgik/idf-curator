extern crate json;
use clap::{Arg, App, SubCommand};
use std::env;
use std::fs;
use std::path::Path;

use md5;

fn print_path(property_path: &std::string::String) {
    let path = Path::new(&property_path);
    let parent = path.parent().unwrap().to_str();
    print!("{}", parent.unwrap());
}

fn main() {
    let idf_tools_path_env = "IDF_TOOLS_PATH";

    let idf_tools_path = env::var(idf_tools_path_env).unwrap_or_else(|e| {
        panic!("could not find {}: {}", idf_tools_path_env, e)
    });

    let content = fs::read_to_string(idf_tools_path + "/esp_idf.json")
    .expect("Failure");
    let parsed2 = json::parse(&content.to_string()).unwrap();

    let matches = App::new("My Test Program")
    .version("0.0.3")
    .author("Juraj Michalek <juraj.michalek@espressif.com>")
    .about("Maintain ESP-IDF installations")
    .subcommand(SubCommand::with_name("get")
        .arg(Arg::with_name("property")
            .short("p")
            .long("property")
            .takes_value(true)
            .help("Path to ESP-IDF installation"))
        .arg(Arg::with_name("idf-path")
            .short("i")
            .long("idf-path")
            .takes_value(true)
            .help("Path to ESP-IDF installation"))
    )
    .subcommand(SubCommand::with_name("add")
        .arg(Arg::with_name("idf-path")
                .short("i")
                .long("idf-path")
                .takes_value(true)
                .help("Path to ESP-IDF installation"))
        .arg(Arg::with_name("python")
                .short("p")
                .long("python")
                .takes_value(true)
                .help("Full path to Python interpreter binary"))
        .arg(Arg::with_name("git")
                .short("g")
                .long("git")
                .takes_value(true)
                .help("Full path to Git binary"))
        .arg(Arg::with_name("idf-version")
                .short("x")
                .long("idf-version")
                .takes_value(true)
                .help("ESP-IDF version"))

    )
    .subcommand(SubCommand::with_name("rm"))
    .subcommand(SubCommand::with_name("inspect"))
    .get_matches();


    if let Some(matches) = matches.subcommand_matches("get") {
        let property_name = matches.value_of("property").unwrap();
        let idf_path = matches.value_of("idf-path");
        if idf_path != None {
            let idf_path_with_slash = format!("{}/", idf_path.unwrap().replace("\\","/"));
            let digest = md5::compute(idf_path_with_slash);
            let idf_id = format!("esp-idf-{:x}", digest);
            let property_path = &parsed2["idfInstalled"][idf_id][property_name].to_string();
            print_path(property_path);
        } else {
            let property_path = &parsed2[property_name].to_string();
            print_path(property_path);
        }
    } else if let Some(_) = matches.subcommand_matches("inspect") {
        println!("{}", &content);
    }
}
