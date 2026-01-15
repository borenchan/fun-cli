use crate::error::CliError;
use crate::impls::handlers::CommandHandler;
use clap::Parser;
use reqwest::blocking::Client;
use serde::Deserialize;
use url::form_urlencoded;

#[derive(Debug, Parser)]
pub struct WeatherHandler {
    #[arg(required = true, help = "输入你想查询的城市名称")]
    city: String,

    #[arg(short, long, default_value_t = 1, help = "输入查询范围，默认当天")]
    mode: u8,
}

impl WeatherHandler {
    pub fn new(city: String, mode: u8) -> Self {
        Self { city, mode }
    }

    pub fn get_now_weather(&self, city: &str) -> Result<WeatherResult, Box<dyn std::error::Error>> {
        let client = Client::new();
        let weather_result = HeFengWeather::new(&client).get_now_weather(city)?;
        Ok(weather_result)
    }
}

impl CommandHandler for WeatherHandler {
    fn run(&self) -> Result<(), CliError> {
        println!("🔊 正在获取天气信息...城市：{}", self.city);
        println!();
        if self.mode == 1 {
            let result = self.get_now_weather(&self.city)?;
            println!("⏰ 数据采集时间: {}", result.time);
            println!("🌡️ 温度: {}", result.temperature);
            println!("🌡️ 体感温度: {}", result.feels_like);
            println!("🌡️ 湿度: {}", result.humidity);
            println!("📒 描述: {}", result.text);
            println!("🌬️ 风向: {}", result.wind_dir);
            println!("🌬️ 风力等级: {}", result.wind_scale);
            println!("🌬️ 风速: {}", result.wind_speed);
            println!("🌧️ 过去1小时降水量: {}", result.precip);
            println!("☁️ 大气压强: {}", result.pressure);
            println!("👀 能见度: {}", result.visibility);
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize, Default)]
struct WeatherResult {
    #[serde(rename = "obsTime", default)]
    time: String, //数据采集时间

    #[serde(rename = "temp", default)]
    temperature: String, //温度

    #[serde(rename = "feelsLike", default)]
    feels_like: String, //体感温度

    #[serde(default)]
    text: String, //天气描述

    #[serde(rename = "windDir", default)]
    wind_dir: String, //风向

    #[serde(rename = "windScale", default)]
    wind_scale: String, //风力等级

    #[serde(rename = "windSpeed", default)]
    wind_speed: String, //风速 公里/小时

    #[serde(default)]
    humidity: String, //湿度  百分比数值

    #[serde(default)]
    precip: String, //过去1小时降水量  毫米

    #[serde(default)]
    pressure: String, // 大气压强/百帕

    #[serde(rename = "vis", default)]
    visibility: String, //能见度 公里
}
struct HeFengWeather<'a> {
    client: &'a Client,
}
impl<'a> HeFengWeather<'a> {
    const HEFENG_GEO_API_URL: &'static str =
        "https://mu4y3j6egv.re.qweatherapi.com/geo/v2/city/lookup?location={city}&key={apiKey}&gzip=n";
    const HEFENG_API_URL: &'static str =
        "https://mu4y3j6egv.re.qweatherapi.com/v7/weather/now?location={location}&key={apiKey}&gzip=n";
    const HEFENG_API_KEY: &'static str = "1d7b188237fc43c5b83c12f1ed996da8";
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }
    /// 获取实时天气信息
    pub fn get_now_weather(&self, city: &str) -> Result<WeatherResult, Box<dyn std::error::Error>> {
        let encoded_city = form_urlencoded::byte_serialize(city.as_bytes()).collect::<String>();
        let geo_api = Self::HEFENG_GEO_API_URL
            .replace("{city}", &encoded_city)
            .replace("{apiKey}", Self::HEFENG_API_KEY);
        let resp = self.client.get(geo_api).send()?.error_for_status()?;
        // eprintln!("{:?}", resp.text().unwrap());
        let res: serde_json::Value = resp.json()?;
        if let Some(locations) = res["location"].as_array() {
            if let Some(loc) = locations.first() {
                let api_url = Self::HEFENG_API_URL
                    .replace("{location}", loc["id"].as_str().unwrap())
                    .replace("{apiKey}", Self::HEFENG_API_KEY);
                let resp = self.client.get(api_url).send()?.error_for_status()?;
                let res: serde_json::Value = resp.json()?;
                let mut wr: WeatherResult = serde_json::from_value(res["now"].clone())?;
                wr.time = wr.time.replace("T", " ").replace("+08:00", "");
                wr.temperature = wr.temperature + "℃";
                wr.feels_like = wr.feels_like + "℃";
                wr.wind_speed = wr.wind_speed + "km/h";
                wr.wind_scale = wr.wind_scale + "级";
                wr.humidity = wr.humidity + "%";
                wr.precip = wr.precip + "mm";
                wr.pressure = wr.pressure + "hPa";
                wr.visibility = wr.visibility + "km";
                return Ok(wr);
            }
        }
        Err(Box::from("城市名称输入有误！"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_weather() {
        let result = HeFengWeather::new(&Client::new())
            .get_now_weather("九江")
            .expect("获取天气信息失败！");
        println!("{:?}", result);
    }
}
