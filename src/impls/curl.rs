use crate::error::CliError;
use crate::impls::handlers::CommandHandler;
use clap::{Parser, ValueEnum, ValueHint};
use reqwest::blocking::Client;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Debug, Parser)]
pub struct CurlHandler {
    #[arg(
        value_hint = ValueHint::Url,
        value_parser = parse_url,
        required = true,
        help = "请求的URL"
    )]
    url: String,

    #[arg(
        short='X',
        long,
        value_enum,
        default_value_t = HttpMethod::Get,
        help = "HTTP请求方式"
    )]
    method: HttpMethod,

    #[arg(
        short='H',
        long,
        help = "请求的Header",
        value_parser = parse_header
    )]
    headers: Option<Vec<(String, String)>>,

    #[arg(short, long, help = "请求的Body")]
    data: Option<String>,

    #[arg(
        short,
        long,
        value_hint = ValueHint::FilePath,
        help = "保存响应的输出文件"
    )]
    output: Option<PathBuf>,
}

// 枚举类型：HTTP 方法
#[derive(Debug, Clone, ValueEnum)]
enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Options,
}
fn parse_url(url: &str) -> Result<String, String> {
    if url.starts_with("http://") || url.starts_with("https://") {
        Ok(url.to_string())
    } else {
        Err(format!("Invalid URL: {}", url))
    }
}

fn parse_header(header: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = header.split(":").collect();
    if parts.len() != 2 {
        return Err(format!("Invalid header format: {}", header));
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}

impl CommandHandler for CurlHandler {
    fn run(&self) -> Result<(), CliError> {
        let client = Client::new();
        println!("🌍：请求URL: {}", self.url);
        let mut req = match self.method {
            HttpMethod::Get => client.get(&self.url),
            HttpMethod::Post => client.post(&self.url),
            HttpMethod::Put => client.put(&self.url),
            HttpMethod::Patch => client.patch(&self.url),
            HttpMethod::Delete => client.delete(&self.url),
            HttpMethod::Options => client.request(reqwest::Method::OPTIONS, &self.url),
        };
        if let Some(headers) = &self.headers {
            println!("请求头：");
            for (key, value) in headers.iter() {
                println!("{}:{}", key, value);
                req = req.header(key, value);
            }
            // headers.iter().for_each(|(key, value)| println!("{}:{}", key, value));
        }
        if let Some(data) = &self.data {
            req = req.body(data.to_string());
            println!("请求体：{}", data);
        }
        let start = Instant::now();

        if let Some(output) = &self.output {
            let resp = req.send()?.error_for_status()?;
            let elapsed = start.elapsed().as_millis();
            println!("请求耗时：{}ms", elapsed);
            let content = resp.text()?;
            println!("保存响应到文件：{}", output.display());
            std::fs::write(output, content)?;
        } else {
            let resp = req.send()?.error_for_status()?;
            let elapsed = start.elapsed().as_millis();
            println!("请求耗时：{}ms", elapsed);
            println!("状态码：{}", resp.status());
            println!("响应头：");
            for x in resp.headers() {
                if let Ok(val) = x.1.to_str() {
                    println!("{}:{}", x.0, val)
                }
            }
            println!("✅ 响应体：");
            println!("{}", resp.text()?);
        }
        Ok(())
    }
}
