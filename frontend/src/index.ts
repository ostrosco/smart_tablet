import './style.css';
import * as clock from './clock/clock';
import { GlobalData } from './globalData';

// Main

console.log('Smart Tablet main script executing.');

let globalData = new GlobalData();
let content = new clock.ClockPanel(globalData);

content.setUp();

window.requestAnimationFrame(updateTimeCallback);

// Helper functions

function updateTimeCallback(): void {
  content?.animationFrameTick();
  window.requestAnimationFrame(updateTimeCallback);
}


try {
  globalData.apiKey = require('./openWeatherMapApiKey.json');
  console.log(globalData.apiKey.key);
} catch {
  console.log('Open Weather Map API key not available.');
}

if (navigator.geolocation) {
  navigator.geolocation.getCurrentPosition(setPositionInClock);
} else {
  console.log("Location not supported.");
}

export function setPositionInClock(position: GeolocationPosition): void {
  // eventually, we could reverse geocode the lat and long and get a location that's more meaningful to humans
  globalData.location = position;

  // query weather (eventually refactor the control flow for get location -> get weather)
  getWeather(position);
}

export async function getWeather(pos: GeolocationPosition): Promise<void> {
  if (!globalData.apiKey) {
    console.log("Aborting weather query: no API key.");
    return;
  }

  const queryString = `http://api.openweathermap.org/data/2.5/weather?lat=${pos.coords.latitude}&lon=${pos.coords.longitude}&appid=${globalData.apiKey.key}&units=imperial`;

  var response;
  var responseJson;

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

  globalData.weather = responseJson;
}