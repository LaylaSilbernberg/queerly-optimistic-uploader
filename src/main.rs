slint::include_modules!();
use slint::SharedString;
use std::error::Error;
use std::path::PathBuf;
type SyncResponseResult = Result<reqwest::Response, tokio::io::Error>;
mod csv_record;

#[tokio::main]
async fn main() {
    let handle = tokio::runtime::Handle::current();
    let app = QueerlyApp::new().unwrap();
    on_click_path(app.clone_strong());
    on_click_uploader(app.clone_strong(), handle.clone());

    let _ = app.run();
}
fn run_reader(path: &str) -> Result<Vec<csv_record::CsvRecord>, Box<dyn Error>> {
    let mut csv_vec: Vec<csv_record::CsvRecord> = Vec::new();
    let Ok(mut csv_reader) = csv::Reader::from_path(path) else {
        return Err(From::from("failure to read csv"));
    };
    for record in csv_reader.deserialize() {
        let csv: csv_record::CsvRecord = record?;
        csv_vec.push(csv);
    }
    Ok(csv_vec)
}

async fn find_file() -> PathBuf {
    let join = slint::spawn_local(async move {
        let file = rfd::AsyncFileDialog::new()
            .set_directory(dirs::home_dir().expect("HOME should be set"))
            .add_filter("csv", &["csv"])
            .set_title("Pick a csv file")
            .pick_file()
            .await
            .expect("Something went wrong parsing your file structure");

        return file.path().to_path_buf();
    });

    join.unwrap().await.to_owned()
}

fn on_click_path(app: QueerlyApp) {
    let app_weak = app.as_weak();
    app.on_click_path(move || {
        let app = app_weak.upgrade().unwrap();
        let _ = slint::spawn_local(async move {
            let path = find_file().await;
            let Some(file) = path.to_str() else {
                return;
            };
            let file_path = SharedString::from(file);
            app.set_path(file_path);
            app.set_visible_text(true);
        });
    });
}
fn on_click_uploader(app: QueerlyApp, handle: tokio::runtime::Handle) {
    let client = reqwest::Client::new();
    let app_weak = app.as_weak();
    app.on_click_uploader(move || {
        let handle_clone = handle.clone();
        let Some(app) = app_weak.upgrade() else {
            return;
        };
        let Ok(app_url) = reqwest::Url::parse(app.get_url().as_str()) else {
            return println!("Url incorrectly formatted");
        };
        let csv_records = match run_reader(app.get_path().as_str()) {
            Ok(result) => result,
            Err(error) => return println!("Error: {error}"),
        };
        let csv_dtos: Vec<csv_record::CsvDto> = csv_records
            .into_iter()
            .map(|record| record.into_dto())
            .collect();
        for record in csv_dtos {
            let url = app_url.clone();
            let client = client.clone();
            let loop_handle = handle_clone.clone();
            let joiner = slint::spawn_local(async move {
                let join_handle: tokio::task::JoinHandle<SyncResponseResult> =
                    loop_handle.spawn(async move {
                        let Ok(json) = serde_json::to_string(&record) else {
                            return Err(tokio::io::Error::new(
                                tokio::io::ErrorKind::InvalidInput,
                                "json is incorrect",
                            ));
                        };
                        let Ok(result) = client.post(url).body(json).send().await else {
                            return Err(tokio::io::Error::new(
                                std::io::ErrorKind::AddrNotAvailable,
                                "Something went wrong with the connection",
                            ));
                        };

                        Ok(result)
                    });
                if let Ok(result) = join_handle.await {
                    let Ok(nested_result) = result else {
                        return eprintln!("json is malformed");
                    };
                    if nested_result.status() != reqwest::StatusCode::OK {
                        return eprintln!(
                            "Something went wrong! Status: {}",
                            nested_result.status()
                        );
                    }
                    println!("Post successful: status = {}", nested_result.status())
                }
            });

            match joiner {
                Ok(_result) => {}
                Err(error) => return println!("Error: {error}"),
            }
        }
    });
}
