import * as dayjs from "dayjs";

export class Weather {
  current_weather: CurrentWeather;
  forecast: Forecast[];

  constructor(data: any) {
    this.current_weather = data.current_weather;

    this.forecast = data.forecast;

    for (const f of this.forecast) {
      f.date = dayjs(f.date, 'YYYY-MM-DD').toDate();
    }
  }
}

export class CurrentWeather {
  temp: number;
  humidity: number;
  description: string;
}

export class Forecast {
  date: Date;
  min_temp: number;
  max_temp: number;
  humidity: number;
  rain_chance: number;
  cloudiness: number;
  description: string;
}