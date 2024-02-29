#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use eframe::egui::Visuals;
use serde_json::Value;
use std::str::FromStr;

#[derive(Debug)]
struct Location {
    latitude: String,
    longitude: String,
}

impl Location {
    fn get_location(address: String) -> Location {
        let info = geolocation::find(&address).unwrap();
        Location {
            latitude: info.latitude,
            longitude: info.longitude,
        }
    }
}

fn request(url: &str) -> Result<serde_json::Value, ureq::Error> {
    let body: String = ureq::get(url)
        .set("Example-Header", "header value")
        .call()?
        .into_string()?;
    let v: Value = serde_json::from_str(&body).expect("data not found");
    Ok(v)
}

fn get_ip() -> String {
    let resp = reqwest::blocking::get("https://ifconfig.me").expect("reason").text();
    match resp {
        Ok(res) => res,
        Err(_error) => panic!("error getting ip"),
    }
}

#[derive(Debug)]
struct Output {
    start_time: String,
    end_time: String,
    is_day_time: bool,
    temp: i64,
    temp_unit: String,
    percip_percent: u64,
    //dewpoint_degree: i64,
    humidity_percent: u64,
    wind_speed: String,
    wind_direction: String,
    short_forecast: ShortForecast,
    //test_enum: ShortForecast,
}

//TODO add other possible weather enums(add to impl below and impl for myapp)
#[derive(Debug)]
enum ShortForecast {
    Sunny,
    Clear,
    PartlyCloudy,
    MostlyCloudy,
    MostlySunny,
    PartlySunny,
}

//required to convert from string into enum
impl FromStr for ShortForecast {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "\"Sunny\""  => Ok(ShortForecast::Sunny),
            "\"Clear\""  => Ok(ShortForecast::Clear),
            "\"Partly Cloudy\""  => Ok(ShortForecast::PartlyCloudy),
            "\"Mostly Cloudy\"" => Ok(ShortForecast::MostlyCloudy),
            "\"Partly Sunny\"" => Ok(ShortForecast::PartlySunny),
            "\"Mostly Sunny\"" => Ok(ShortForecast::MostlySunny),
            _      => Err(()),
        }
    }
}

struct MyApp {
     data: Output,
}

impl MyApp {
    fn new(data: Output) -> Self {
        Self {
            data
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            //ui.heading("My egui Application");

            //TODO change outputs to pictures
            match self.data.short_forecast {
                ShortForecast::Clear => ui.add(egui::Image::new(egui::include_image!("misty.png")).max_height(32.0).max_width(32.0)),
                ShortForecast::Sunny => ui.add(egui::Image::new(egui::include_image!("sun.png")).max_height(32.0).max_width(32.0)),
                ShortForecast::PartlyCloudy => ui.add(egui::Image::new(egui::include_image!("cloud.png")).max_height(32.0).max_width(32.0)),
                ShortForecast::MostlyCloudy => ui.add(egui::Image::new(egui::include_image!("cloud-2.png")).max_height(32.0).max_width(32.0)),
                ShortForecast::PartlySunny => ui.add(egui::Image::new(egui::include_image!("cloudy.png")).max_height(32.0).max_width(32.0)),
                ShortForecast::MostlySunny => ui.add(egui::Image::new(egui::include_image!("sun-2.png")).max_height(32.0).max_width(32.0)),
            };

            //ui.label(format!("{}", self.data.short_forcast));
            ui.label(format!("current temp {}{}", self.data.temp, self.data.temp_unit));
            ui.label(format!("wind speed {}, direction {}", self.data.wind_speed, self.data.wind_direction));
            ui.horizontal(|ui| {
                ui.label(format!("percipitation {}%", self.data.percip_percent));
                ui.label(format!("humidity {}%", self.data.humidity_percent));
            });
            ui.vertical(|ui| {
                ui.label(format!("start {}", self.data.start_time));
                ui.label(format!("end {}", self.data.end_time));
            });


            //TODO
            if self.data.is_day_time {
                ui.style_mut().visuals = Visuals::light();
            } else {
                ui.style_mut().visuals = Visuals::dark();
            }


            // ui.horizontal(|ui| {
            //     let name_label = ui.label("Your name: ");
            //     ui.text_edit_singleline(&mut self.name)
            //         .labelled_by(name_label.id);
            // });
            // ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
            // if ui.button("Increment").clicked() {
            //     self.age += 1;
            // }
            // ui.label(format!("Hello '{}', age {}", self.name, self.age));

            // ui.image(egui::include_image!(
            //     "../../../crates/egui/assets/ferris.png"
            // ));
        });
    }
}

fn main() -> Result<(), eframe::Error>{
    //gets location for url
    let loc = Location::get_location(get_ip());
    let url = format!("https://api.weather.gov/points/{},{}", loc.latitude, loc.longitude);

    //gets first body that includes hourly url
    let body = request(&url);
    let hourly = &body.unwrap()["properties"]["forecastHourly"];

    //sets second body containing weather data and sets data
    let body2 = request(hourly.as_str().expect("no url")).unwrap();
    let visual = Output {
        start_time: (body2["properties"]["periods"][0]["startTime"]).to_string(),
        end_time: (body2["properties"]["periods"][0]["endTime"]).to_string(),
        is_day_time: (body2["properties"]["periods"][0]["isDaytime"]).as_bool().expect("err"),
        temp: (body2["properties"]["periods"][0]["temperature"]).as_i64().expect("err"),
        temp_unit: (body2["properties"]["periods"][0]["temperatureUnit"]).to_string(),
        percip_percent: (body2["properties"]["periods"][0]["probabilityOfPrecipitation"]["value"]).as_u64().expect("err"),
        //dewpoint_degree: (body2["properties"]["periods"][0]["dewpoint"]["value"]).as_i64().expect("err"),
        humidity_percent: (body2["properties"]["periods"][0]["relativeHumidity"]["value"]).as_u64().expect("err"),
        wind_speed: (body2["properties"]["periods"][0]["windSpeed"]).to_string(),
        wind_direction: (body2["properties"]["periods"][0]["windDirection"]).to_string(),
        //short_forecast: (body2["properties"]["periods"][0]["shortForecast"]).to_string(),
        short_forecast: FromStr::from_str(&(body2["properties"]["periods"][0]["shortForecast"]).to_string()).unwrap()
    };

    //creates gui
    //env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Weather App",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            //Box::<MyApp>::default()
            Box::new(MyApp::new(visual))
        }),
    )
}