import { Weather } from "./api-types/weather";

export class GlobalData {
  apiKey?: {key: string};

  lat?: number;
  lon?: number;
  
  weather?: Weather;

  theme?: string;
}