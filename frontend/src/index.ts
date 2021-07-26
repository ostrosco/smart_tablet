import './style.css';
import * as clock from './clock/clock';
import { GlobalData } from './globalData';
import { Weather } from './api-types/weather';

// Main

console.log('Smart Tablet main script executing.');

const globalData = new GlobalData();
const content = new clock.ClockPanel(globalData);

getLocation();
getWeather();

content.setUp();

window.requestAnimationFrame(updateTimeCallback);


function updateTimeCallback(): void {
  content.animationFrameTick();
  window.requestAnimationFrame(updateTimeCallback);
}

// Helper functions

export async function getWeather(): Promise<void> {
  const queryString = "http://localhost:8080/weather";

  let response: Response;
  let responseJson: any;

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
    responseJson = await response.json();
    console.log('Converted successfully');
  }
  catch (ex) {
    console.log("Exception caught converting weather response to JSON:");
    console.log(ex);
    return;
  }

  globalData.weather = new Weather(responseJson);
}

export async function getLocation(): Promise<void> {
  const queryString = "http://localhost:8080/settings";

  let response: Response;
  let responseJson: any;

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
    responseJson = await response.json();
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