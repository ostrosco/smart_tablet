export class Settings {
  weather_settings: WeatherSettings;
  news_settings: NewsSettings;
}

export class WeatherSettings {
  weather_source: string;
  temp_unhits: string;
  polling_rate: number;
  api_key: string;
  lat: number;
  lon: number;
}

export class NewsSettings {
  polling_rate: number;
  news_sources: NewsSource[];
}

export class NewsSource {
  Rss: string;
}