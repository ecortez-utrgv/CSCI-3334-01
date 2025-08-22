use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
    sync::{Arc, Mutex, mpsc},
    thread,
    time::Instant,
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ureq::{Agent, AgentBuilder};

#[derive(Debug)]
struct WebsiteStatus {
    url: String,
    status: Result<u16, String>,
    response_time: std::time::Duration,
    timestamp: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct Config {
    worker_threads: usize,
    request_timeout_secs: u64,
    max_retries: u32,
    log_file: String,
}

impl Config {
    pub fn from_file(path: &str) -> std::io::Result<Config> {
        let file = File::open(path)?;
        serde_json::from_reader(file)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}

struct Job {
    url: String,
    max_retries: u32,
}

fn load_urls(path: &str) -> std::io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut urls = Vec::new();
    for line in reader.lines() {
        let trimmed = line?.trim().to_string();
        if !trimmed.is_empty() {
            urls.push(trimmed);
        }
    }
    Ok(urls)
}

fn build_agent(timeout: std::time::Duration) -> Agent {
    AgentBuilder::new()
        .timeout_connect(timeout)
        .timeout_read(timeout)
        .timeout_write(timeout)
        .build()
}

fn check_once(agent: &Agent, url: &str) -> Result<u16, String> {
    let resp = agent.get(url).call();
    match resp {
        Ok(r) => Ok(r.status()),
        Err(e) => match e {
            ureq::Error::Status(code, _) => Ok(code),
            ureq::Error::Transport(te) => Err(format!("transport error: {}", te)),
        },
    }
}

fn worker_thread(
    id: usize,
    agent: Agent,
    jobs_rx: Arc<Mutex<mpsc::Receiver<Job>>>,
    results_tx: mpsc::Sender<WebsiteStatus>,
) {
    loop {
        let job = {
            let rx = jobs_rx.lock().unwrap();
            rx.recv()
        };

        let job = match job {
            Ok(job) => job,
            Err(_) => break,
        };

        let mut last_result: Result<u16, String> = Err("unattempted".into());
        let mut last_rt = std::time::Duration::from_millis(0);

        let attempts = job.max_retries.saturating_add(1);
        for _ in 0..attempts {
            let start = Instant::now();
            let res = check_once(&agent, &job.url);
            last_rt = start.elapsed();
            match &res {
                Ok(_) => {
                    last_result = res;
                    break;
                }
                Err(_) => {
                    last_result = res;
                }
            }
        }

        let status = WebsiteStatus {
            url: job.url,
            status: last_result,
            response_time: last_rt,
            timestamp: Utc::now(),
        };

        if results_tx.send(status).is_err() {
            eprintln!("[worker-{id}] results receiver dropped; exiting");
            return;
        }
    }
}

fn pretty_print(ws: &WebsiteStatus) {
    match &ws.status {
        Ok(code) => println!(
            "[{}] {} | status={} | rt={:?}",
            ws.timestamp.to_rfc3339(),
            ws.url,
            code,
            ws.response_time
        ),
        Err(err) => println!(
            "[{}] {} | error={} | rt={:?}",
            ws.timestamp.to_rfc3339(),
            ws.url,
            err,
            ws.response_time
        ),
    }
}

fn to_json_line(ws: &WebsiteStatus) -> String {
    let rt_ms = ws.response_time.as_millis();
    let ts = ws.timestamp.to_rfc3339();

    match &ws.status {
        Ok(code) => serde_json::json!({
            "url": ws.url,
            "status": code,
            "response_time_ms": rt_ms,
            "timestamp": ts
        })
        .to_string(),
        Err(err) => serde_json::json!({
            "url": ws.url,
            "status": err,
            "response_time_ms": rt_ms,
            "timestamp": ts
        })
        .to_string(),
    }
}

fn main() -> std::io::Result<()> {
    let config = Config::from_file("config.json")?;
    let urls = load_urls("urls.txt")?;

    if urls.is_empty() {
        eprintln!("No URLs found in urls.txt. Exiting.");
        return Ok(());
    }

    let worker_threads = config.worker_threads.max(1);
    let timeout = std::time::Duration::from_secs(config.request_timeout_secs);
    let max_retries = config.max_retries;

    let mut log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&config.log_file)?;

    writeln!(
        &mut log_file,
        "{{\"run_started\":\"{}\",\"worker_threads\":{},\"timeout_secs\":{},\"max_retries\":{}}}",
        Utc::now().to_rfc3339(),
        worker_threads,
        timeout.as_secs(),
        max_retries
    )?;

    let (jobs_tx, jobs_rx) = mpsc::channel::<Job>();
    let (results_tx, results_rx) = mpsc::channel::<WebsiteStatus>();

    let jobs_rx = Arc::new(Mutex::new(jobs_rx));

    let mut handles = Vec::new();
    for i in 0..worker_threads {
        let agent = build_agent(timeout);
        let rx = Arc::clone(&jobs_rx);
        let tx = results_tx.clone();
        let handle = thread::spawn(move || worker_thread(i, agent, rx, tx));
        handles.push(handle);
    }
    drop(results_tx);

    for url in urls {
        let job = Job { url, max_retries };
        jobs_tx.send(job).unwrap();
    }
    drop(jobs_tx);

    for ws in results_rx {
        pretty_print(&ws);
        let line = to_json_line(&ws);
        writeln!(log_file, "{}", line)?;
    }

    for h in handles {
        let _ = h.join();
    }

    writeln!(
        &mut log_file,
        "{{\"run_finished\":\"{}\"}}",
        Utc::now().to_rfc3339()
    )?;

    Ok(())
}
