use clap::Arg;
use clap_nested::{Command, Commander, MultiCommand};
use std::path::Path;
use std::io::Cursor;
#[cfg(windows)]
use std::collections::HashMap;
use tokio::runtime::Handle;
use std::fs;
use std::io;


#[cfg(windows)]
use core::ptr::null_mut;

#[cfg(windows)]
mod windows;

#[cfg(windows)]
use windows::to_wchar;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[cfg(unix)]
fn get_driver_property(_property_name: String, _filter: String) -> Result<()> {
    Ok(())
}

#[cfg(windows)]
fn get_driver_property(property_name: String, filter: String) -> Result<()> {
    use wmi::*;
    use wmi::Variant;

    let wmi_con = WMIConnection::with_namespace_path("ROOT\\CIMV2", COMLibrary::new()?.into())?;
    let query = format!("SELECT {} FROM Win32_PnPEntity WHERE {}", property_name, filter);
    // println!("Query: {}", query);
    let results: Vec<HashMap<String, Variant>> = wmi_con.raw_query(query).unwrap();

    for driver_item in results {
        match property_name == "*" {
            true => println!("{:#?}", driver_item),
            _ => {
                let property_value = &driver_item[&property_name];

                if let Variant::String(value) = property_value {
                    println!("{}", value)
                }
            }
        }
    }
    Ok(())
}

fn get_installed_driver_property(property_name: String) -> Result<()> {
    // Driver classes: https://docs.microsoft.com/en-us/windows-hardware/drivers/install/system-defined-device-setup-classes-available-to-vendors?redirectedfrom=MSDN
    return get_driver_property(property_name, "ClassGuid=\"{4d36e978-e325-11ce-bfc1-08002be10318}\"".to_string());
}

fn get_missing_driver_property(property_name: String) -> Result<()> {
    // https://stackoverflow.com/questions/11367639/get-a-list-of-devices-with-missing-drivers-using-powershell
    return get_driver_property(property_name, "ConfigManagerErrorCode>0".to_string());
}

async fn fetch_url(url: String, output: String) -> Result<()> {
    let response = reqwest::get(url).await?;
    let mut file = std::fs::File::create(output)?;
    let mut content = Cursor::new(response.bytes().await?);
    std::io::copy(&mut content, &mut file)?;
    Ok(())
}

async fn download_zip(url: String, output: String) -> Result<()> {
    if Path::new(&output).exists() {
        println!("Using cached driver.");
        return Ok(());
    }
    fetch_url(url, output).await
}

fn download_driver(driver_url: String, driver_archive: String) -> Result<()> {
    let handle = Handle::current().clone();
    let th = std::thread::spawn(move || {
        handle.block_on(download_zip(driver_url, driver_archive)).unwrap();
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
                    .takes_value(true)
                    .default_value("*"),
            )
                .arg(
                    Arg::with_name("missing")
                        .short("m")
                        .long("missing")
                        .help("Display missing drivers")
                )
        })
        .runner(|_args, matches| {
            let property_name = matches.value_of("property").unwrap().to_string();
            if matches.is_present("missing") {
                get_missing_driver_property(property_name).unwrap();
            } else {
                get_installed_driver_property(property_name).unwrap();
            }
            Ok(())
        })
}

fn unzip(file_path: String) -> Result<()> {
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

#[cfg(unix)]
fn install_driver(driver_inf: String, driver_url: String, _driver_archive: String) {}

#[cfg(windows)]
fn install_driver(driver_inf: String, driver_url: String, _driver_archive: String) {
    let driver_archive = _driver_archive.clone();
    download_driver(driver_url, _driver_archive).unwrap();
    if !Path::new(&driver_inf).exists() {
        unzip(driver_archive).unwrap();
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

    // let source_inf_filename = to_wchar("tmp/silabser.inf").as_ptr();
    let source_inf_filename = to_wchar(&driver_inf).as_ptr();
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
}

#[cfg(unix)]
fn get_install_runner(_args: &str, _matches: &clap::ArgMatches<'_>) -> std::result::Result<(), clap::Error> {
    Ok(())
}

#[cfg(windows)]
fn get_install_runner(_args: &str, _matches: &clap::ArgMatches<'_>) -> std::result::Result<(), clap::Error> {
    if windows::is_app_elevated() {
        if _matches.is_present("silabs") {
            install_driver("tmp/silabser.inf".to_string(),
                           "https://www.silabs.com/documents/public/software/CP210x_Universal_Windows_Driver.zip".to_string(),
                           "cp210x.zip".to_string());
        }

        if _matches.is_present("ftdi") {
            install_driver("tmp/ftdiport.inf".to_string(),
                           "https://www.ftdichip.com/Drivers/CDM/CDM%20v2.12.28%20WHQL%20Certified.zip".to_string(),
                           "ftdi.zip".to_string());
        }
    } else {
        let mut arguments: Vec<String> = [].to_vec();

        if _matches.is_present("silabs") {
            arguments.push("--silabs".to_string());
        }

        if _matches.is_present("ftdi") {
            arguments.push("--ftdi".to_string());
        }

        if arguments.len() == 0 {
            println!("No driver specified.");
        } else {

            // Based on https://github.com/rust-lang/rustup/pull/1117/files
            println!("Installation requires elevated privileges.");
            println!("Requesting elevation.");
            let current_exe = std::env::current_exe().unwrap().display().to_string();
            let argument_string = arguments.clone().into_iter().map(|i| format!("{} ", i.to_string())).collect::<String>();
            let parameters_string = format!("driver install {}", argument_string);
            let operation = to_wchar("runas");
            let path = to_wchar(&current_exe);
            let parameters = to_wchar(&parameters_string);
            let sw_showminnoactive = 7;

            let result = unsafe {
                // https://docs.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shellexecutew
                winapi::um::shellapi::ShellExecuteW(null_mut(),
                                                    operation.as_ptr(),
                                                    path.as_ptr(),
                                                    parameters.as_ptr(),
                                                    null_mut(),
                                                    sw_showminnoactive)
            };
            println!("{:?}", result);
            // pub fn ShellExecuteA(
            //     hwnd: HWND,
            //     lpOperation: LPCSTR,
            //     lpFile: LPCSTR,
            //     lpParameters: LPCSTR,
            //     lpDirectory: LPCSTR,
            //     nShowCmd: c_int,
            // ) -> HINSTANCE;
        }
    }
    Ok(())
}

pub fn get_install_cmd<'a>() -> Command<'a, str> {
    Command::new("install")
        .description("Install driver - requires elevated privileges")
        .options(|app| {
            app.arg(
                Arg::with_name("ftdi")
                    .short("f")
                    .long("ftdi")
                    .help("Install ftdi driver"),
            )
                .arg(
                    Arg::with_name("silabs")
                        .short("s")
                        .long("silabs")
                        .help("Install Silabs drivers"),
                )
                .arg(
                    Arg::with_name("verbose")
                        .short("w")
                        .long("verbose")
                        .takes_value(false)
                        .help("display diagnostic log after installation"))
        })
        .runner(|_args, matches| get_install_runner(_args, matches)
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
