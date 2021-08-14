import './style.css';
import { ClockPanel } from './clock/clock';
import { WeatherPanel } from './weather/weather';
import { GlobalData } from './globalData';
import { Weather } from './api-types/weather';
import { Settings } from './api-types/settings';
import { ContentPanel } from './contentPanel';

// Main

console.log('Smart Tablet main script executing.');

const globalData = new GlobalData();
let currentPanel: ContentPanel;

getLocation();
getWeather();

changeTabTo('clock');

window.requestAnimationFrame(updateTimeCallback);

document.getElementById("clockMenuButton").onclick = (e: MouseEvent) => changeTabTo('clock');
document.getElementById("weatherMenuButton").onclick = (e: MouseEvent) => changeTabTo('weather');

function updateTimeCallback(): void {
  currentPanel.animationFrameTick();
  window.requestAnimationFrame(updateTimeCallback);
}

function changeTabTo(tab: string) : void {
  if (tab === 'clock') {
    currentPanel?.tearDown();
    currentPanel = new ClockPanel(globalData);
    currentPanel.setUp();
  } else if (tab === 'weather') {
    currentPanel?.tearDown();
    currentPanel = new WeatherPanel(globalData);
    currentPanel.setUp();
  }
}

// Helper functions

export async function getWeather(): Promise<void> {
  const queryString = "http://localhost:8080/weather";

  let response: Response;
  let responseJson: Weather;

  try {
    console.log(`Querying ${queryString} ...`);
    response = await fetch(queryString);
    console.log('Query completed successfully.');
  }
  catch (ex) {
    console.log("Exception caught querying weather:");
    console.log(ex);
    return;
  }

  try {
    console.log('Converting query response to json...');
    responseJson = new Weather(await response.json());
    console.log('Converted successfully');
  }
  catch (ex) {
    console.log("Exception caught converting weather response to JSON:");
    console.log(ex);
    return;
  }

  globalData.weather = responseJson;
}

export async function getLocation(): Promise<void> {
  const queryString = "http://localhost:8080/settings";

  let response: Response;
  let responseJson: Settings;

  try {
    console.log(`Querying ${queryString} ...`);
    response = await fetch(queryString);
    console.log('Query completed successfully.');
  }
  catch (ex) {
    console.log("Exception caught querying weather:");
    console.log(ex);
    return;
  }

  try {
    console.log('Converting query response to json...');
    responseJson = await response.json() as Settings;
    console.log('Converted successfully');
  }
  catch (ex) {
    console.log("Exception caught converting weather response to JSON:");
    console.log(ex);
    return;
  }

  globalData.lat = responseJson.weather_settings.lat;
  globalData.lon = responseJson.weather_settings.lon;
}