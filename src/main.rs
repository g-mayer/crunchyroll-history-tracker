use chrono::{DateTime, Utc};
use crunchyroll_rs::{common::StreamExt, Crunchyroll, MediaCollection};
use dotenvy::dotenv;
use serde_json::json;
use std::env;
use std::{
    collections::HashMap,
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::Path,
};

/// Reads the last cutoff date from `cutoff_date.txt`, if it exists.
fn read_cutoff_date() -> Result<Option<DateTime<Utc>>, io::Error> {
    let filename = "cutoff_date.txt";

    match fs::read_to_string(filename) {
        Ok(contents) => {
            let date_str = contents.trim();
            if date_str.is_empty() {
                return Ok(None);
            }
            match date_str.parse::<DateTime<Utc>>() {
                Ok(date) => Ok(Some(date)),
                Err(_) => Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Invalid date format in cutoff_date.txt",
                )),
            }
        }
        Err(err) => {
            if err.kind() == io::ErrorKind::NotFound {
                return Ok(None);
            }
            Err(err)
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    // Set a limit for how many episodes to query from watch history. Example to query 10 episodes: Some(10)
    let limit: Option<usize> = None;

    let username = env::var("CR_USERNAME").expect("Missing CR_USERNAME in .env");
    let password = env::var("CR_PASSWORD").expect("Missing CR_PASSWORD in .env");

    let crunchyroll = Crunchyroll::builder()
        .login_with_credentials(username, password)
        .await?;

    // Read the last cutoff date from file
    let cutoff_date = match read_cutoff_date() {
        Ok(Some(date)) => {
            println!("Previous cutoff date: {}", date);
            Some(date)
        }
        Ok(None) => {
            println!("⚠️ Warning: No cutoff date found! Proceeding without a cutoff date.");
            None
        }
        Err(err) => {
            eprintln!(
                "❌ Error reading cutoff date: {}. Proceeding without a cutoff date.",
                err
            );
            None
        }
    };

    let mut history_stream = crunchyroll.watch_history();
    let mut title_episode_counts: HashMap<String, u32> = HashMap::new();
    let mut extracted_series_data: HashMap<String, serde_json::Value> = HashMap::new();

    // Get the current UTC time to update the cutoff date at the end
    let script_run_time = Utc::now();
    println!("Script started at: {}", script_run_time);

    let mut total_show_count = 0;

    while let Some(entry_result) = history_stream.next().await {
        if let Some(limit_value) = limit {
            if total_show_count >= limit_value {
                break;
            }
        }

        match entry_result {
            Ok(entry) => {
                let entry_date: DateTime<Utc> = entry.date_played;

                let media_collection = crunchyroll
                    .media_collection_from_id(entry.parent_id)
                    .await?;

                let title = match &media_collection {
                    MediaCollection::Movie(movie) => movie.title.clone(),
                    MediaCollection::Series(series) => series.title.clone(),
                    MediaCollection::Episode(episode) => episode.title.clone(),
                    _ => continue,
                };

                // Stop processing if the entry is before the cutoff date
                if let Some(cutoff) = cutoff_date {
                    if entry_date < cutoff {
                        println!(
                            "Stopping: Show watched before cutoff ({}). Title: {}",
                            cutoff, title
                        );
                        break;
                    }
                }

                // Update episode count
                let episode_count = title_episode_counts.entry(title.clone()).or_insert(0);
                *episode_count += 1;

                // If we haven't extracted this series yet, do it now
                if let MediaCollection::Series(series) = &media_collection {
                    if !extracted_series_data.contains_key(&title) {
                        let series_data = json!({
                            "title": series.title,
                            "slug": series.slug_title,
                            "description": series.description,
                            "extendedDescription": series.extended_description,
                            "episodes": series.episode_count,
                            "seasons": series.season_count,
                            "publisher": series.content_provider.clone().unwrap_or("Unknown".to_string()),
                            "keywords": series.keywords,
                            "posterTall": series.images.poster_tall
                                .get(2)
                                .map(|img| img.source.clone())
                                .unwrap_or("No image available".to_string())
                        });

                        extracted_series_data.insert(title.clone(), series_data);
                    }
                }

                println!("{}: {} episodes watched", title, episode_count);
                total_show_count += 1;
            }
            Err(err) => eprintln!("Error fetching watch history entry: {:?}", err),
        }
    }

    let filename = get_unique_filename("show_data.json");
    let mut file = File::create(&filename)?;

    // Create JSON structure combining series data and episodes watched
    let output_data: Vec<_> = extracted_series_data
        .into_iter()
        .map(|(title, series_data)| {
            let episodes_watched = title_episode_counts.get(&title).cloned().unwrap_or(0);
            json!({
                "series": series_data,
                "episodesWatched": episodes_watched
            })
        })
        .collect();

    writeln!(file, "{}", serde_json::to_string_pretty(&output_data)?)?;

    println!("Extracted data saved to: {}", filename);
    println!("Finished processing! Check {} for results.", filename);

    // Update the cutoff date for next run
    update_cutoff_date(script_run_time)?;

    Ok(())
}

/// Updates the `cutoff_date.txt` with the new script run time.
fn update_cutoff_date(new_cutoff: DateTime<Utc>) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("cutoff_date.txt")?;

    writeln!(file, "{}", new_cutoff)?;
    println!("✅ Updated cutoff date to: {}", new_cutoff);
    Ok(())
}

/// Generates a unique filename by appending a number if the file already exists.
fn get_unique_filename(base_name: &str) -> String {
    let mut counter = 1;
    let mut new_name = format!("{}.json", base_name.trim_end_matches(".json"));

    while Path::new(&new_name).exists() {
        new_name = format!("{}-{}.json", base_name.trim_end_matches(".json"), counter);
        counter += 1;
    }

    new_name
}
