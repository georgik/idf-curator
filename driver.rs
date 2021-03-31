use core::ptr::null_mut;
use clap::Arg;
use clap_nested::{Command, Commander, MultiCommand};
use std::path::Path;
use std::io::Cursor;
use wmi::*;
use std::collections::HashMap;
use wmi::Variant;
use tokio::runtime::Handle;
use std::fs;
use std::io;
use std::ffi::OsStr;
use std::os::windows::prelude::*;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;


fn get_driver_property(property_name: String, query: String) -> Result<()>  {
    let wmi_con = WMIConnection::with_namespace_path("ROOT\\CIMV2", COMLibrary::new()?.into())?;
    let results: Vec<HashMap<String, Variant>> = wmi_con.raw_query(query).unwrap();
    for driver_item in results {
        let property_value = &driver_item[&property_name];

        if let Variant::String(value) = property_value {
            print!("{}", value )
        }

    }
    Ok(())
}

fn get_installed_driver_property(property_name: String) -> Result<()> {
    // Driver classes: https://docs.microsoft.com/en-us/windows-hardware/drivers/install/system-defined-device-setup-classes-available-to-vendors?redirectedfrom=MSDN
    return get_driver_property(property_name, "SELECT * FROM Win32_PnPEntity WHERE ClassGuid=\"{4d36e978-e325-11ce-bfc1-08002be10318}\"".to_string());
}

fn get_missing_driver_property(property_name: String) -> Result<()> {
    // https://stackoverflow.com/questions/11367639/get-a-list-of-devices-with-missing-drivers-using-powershell
    return get_driver_property(property_name, "SELECT * FROM Win32_PnPEntity WHERE ConfigManagerErrorCode>0".to_string());
}

async fn fetch_url(url: String) -> Result<()> {
    let response = reqwest::get(url).await?;
    let mut file = std::fs::File::create("driver.zip")?;
    let mut content =  Cursor::new(response.bytes().await?);
    std::io::copy(&mut content, &mut file)?;
    Ok(())
}

async fn download_zip(url: String, output: String) -> Result<()> {
    if Path::new(&output).exists() {
        println!("Using cached driver.");
        return Ok(());
    }
    fetch_url(url).await
}

fn download_driver() -> Result<()> {
    let driver_archive = "driver.zip".to_string();
    let handle = Handle::current().clone();
    let th = std::thread::spawn(move || {
        handle.block_on(download_zip("https://www.silabs.com/documents/public/software/CP210x_Universal_Windows_Driver.zip".to_string(), driver_archive)).unwrap();
    });
    Ok(th.join().unwrap())
}

pub fn get_cmd<'a>() -> Command<'a, str> {
    Command::new("get")
        .description("Get information about drivers")
        .options(|app| {
            app.arg(
                Arg::with_name("property")
                    .short("p")
                    .long("property")
                    .help("Filter result for property name")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("missing")
                    .short("m")
                    .long("missing")
                    .help("Display missing drivers")
            )
        })
        .runner(|_args, matches| {
            let property_name =  matches.value_of("property").unwrap().to_string();
            if matches.is_present("missing") {
                get_missing_driver_property(property_name).unwrap();
            } else {
                get_installed_driver_property(property_name).unwrap();
            }
            Ok(())
        })
}

fn unzip(file_path:String) -> Result<()> {
    let file_name = std::path::Path::new(&file_path);
    let file = fs::File::open(&file_name).unwrap();

    let mut archive = zip::ZipArchive::new(file).unwrap();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let file_outpath = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };

        // Add path prefix to extract the file
        let mut outpath = std::path::PathBuf::new();
        outpath.push("tmp/");
        outpath.push(file_outpath);

        {
            let comment = file.comment();
            if !comment.is_empty() {
                println!("File {} comment: {}", i, comment);
            }
        }

        if (&*file.name()).ends_with('/') {
            println!("File {} extracted to \"{}\"", i, outpath.display());
            fs::create_dir_all(&outpath).unwrap();
        } else {
            println!(
                "File {} extracted to \"{}\" ({} bytes)",
                i,
                outpath.display(),
                file.size()
            );
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(&p).unwrap();
                }
            }
            let mut outfile = fs::File::create(&outpath).unwrap();
            io::copy(&mut file, &mut outfile).unwrap();
        }
    }
    Ok(())
}

fn to_wchar(str : &str) -> Vec<u16> {
    OsStr::new(str).encode_wide().chain(Some(0).into_iter()).collect()
}

fn get_runner(_args:&str, _matches:&clap::ArgMatches<'_>)  -> std::result::Result<(), clap::Error> {
    download_driver().unwrap();
    if !Path::new("tmp/silabser.inf").exists() {
        unzip("driver.zip".to_string()).unwrap();
    }

    // Reference: https://github.com/microsoft/Windows-driver-samples/tree/master/setup/devcon
    // SetupCopyOEMInf(SourceInfFileName,
    //     NULL,
    //     SPOST_PATH,
    //     0,
    //     DestinationInfFileName,
    //     ARRAYSIZE(DestinationInfFileName),
    //     NULL,
    //     &DestinationInfFileNameComponent))
    // Rust: https://docs.rs/winapi/0.3.9/winapi/um/setupapi/fn.SetupCopyOEMInfW.html

    let source_inf_filename = to_wchar("tmp/silabser.inf").as_ptr();
    let mut destination_inf_filename_vec: Vec<u16> = Vec::with_capacity(255);
    let destination_inf_filename = destination_inf_filename_vec.as_mut_ptr();
    let destination_inf_filename_len = 254;
    let mut v: Vec<u16> = Vec::with_capacity(255);
    let mut a: winapi::um::winnt::PWSTR = v.as_mut_ptr();
    unsafe {
    let result = winapi::um::setupapi::SetupCopyOEMInfW(
        source_inf_filename,
        null_mut(),
        winapi::um::setupapi::SPOST_PATH,
        0,
        destination_inf_filename,
        destination_inf_filename_len,
        null_mut(),
        &mut a as *mut _);
        println!("{:#}", result);
    }

    Ok(())
}

pub fn get_install_cmd<'a>() -> Command<'a, str> {
    Command::new("install")
        .description("Install driver")
        .options(|app| {
            app.arg(
                Arg::with_name("installer")
                    .short("e")
                    .long("installer")
                    .help("Path to installer binary"),
            )
            .arg(
                Arg::with_name("interactive")
                    .short("i")
                    .long("interactive")
                    .help("Run installation in interactive mode"),

            )
            .arg(
                Arg::with_name("verbose")
                    .short("w")
                    .long("verbose")
                    .takes_value(false)
                    .help("display diagnostic log after installation"))
        })
        .runner(|_args,matches|get_runner(_args, matches)
        )
}


pub fn get_multi_cmd<'a>() -> MultiCommand<'a, str, str> {
    let multi_cmd: MultiCommand<str, str> = Commander::new()
    .add_cmd(get_cmd())
    .add_cmd(get_install_cmd())
    .into_cmd("driver")

    // Optionally specify a description
    .description("Detection of Antivirus and handling exception registration.");

    return multi_cmd;
}
