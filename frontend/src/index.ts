import * as dayjs from 'dayjs';
import * as advancedFormat from 'dayjs/plugin/advancedFormat';
import './style.css';

var apiKey: {key: string} | undefined;

try {
  apiKey = require('./openWeatherMapApiKey.json');
  console.log(apiKey.key);
} catch {
  console.log('Open Weather Map API key not available.');
}

dayjs.extend(advancedFormat);

if (navigator.geolocation) {
  navigator.geolocation.getCurrentPosition(setPositionInClock);
} else {
  console.log("Location not supported.");
}

function setPositionInClock(position: GeolocationPosition): void {
  // eventually, we could reverse geocode the lat and long and get a location that's more meaningful to humans
  const lat = position.coords.latitude;
  const long = position.coords.longitude;
  const NSStr = lat >= 0 ? 'N' : 'S';
  const EWStr = long >= 0 ? 'E' : 'W';
  document.getElementById("clock-location").innerHTML = `(${Math.abs(long).toFixed(2)}&#176;${EWStr}, ${Math.abs(lat).toFixed(2)}&#176;${NSStr})`;

  // query weather (eventually refactor the control flow for get location -> get weather)
  getWeather(position);
}

async function getWeather(pos: GeolocationPosition): Promise<void> {
  if (!apiKey) {
    console.log("Aborting weather query: no API key.");
    return;
  }

  const queryString = `http://api.openweathermap.org/data/2.5/weather?lat=${pos.coords.latitude}&lon=${pos.coords.longitude}&appid=${apiKey.key}&units=imperial`;

  var response;
  var responseJson;

  try {
    response = await fetch(queryString);
  }
  catch (ex) {
    console.log("Exception caught querying weather:");
    console.log(ex);
  }

  try {
    responseJson = await response.json();
  }
  catch (ex) {
    console.log("Exception caught converting weather response to JSON:");
    console.log(ex);
  }

  document.getElementById("clock-temperature").innerHTML = `${Math.round(responseJson.main.temp)}&#176;`
  document.getElementById("clock-weather").innerHTML = responseJson.weather[0].description;
}

function updateTime() {
  var currentTime = dayjs();
  document.getElementById("clock-time").innerHTML = currentTime.format('h:mm:ss A');
  document.getElementById("clock-day").innerHTML = currentTime.format('dddd');
  document.getElementById("clock-date").innerHTML = currentTime.format('MMMM Do');
}

let updateTimeCallback = () => {
  updateTime();
  window.requestAnimationFrame(updateTimeCallback);
}

window.requestAnimationFrame(updateTimeCallback);
