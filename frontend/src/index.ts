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

changeTabTo('clock');

window.requestAnimationFrame(frameCallback);

document.getElementById("clockMenuButton").onclick = (e: MouseEvent) => changeTabTo('clock');
document.getElementById("weatherMenuButton").onclick = (e: MouseEvent) => changeTabTo('weather');

// WebSocket stuff

let socket = new WebSocket("ws://localhost:9000");

socket.onopen = () => {
  console.log("socket opened");
};

socket.onmessage = (event) => {
  console.log("Received message:");
  console.log(event);

  const messageData = JSON.parse(event.data);
  console.log(messageData);

  if (messageData.hasOwnProperty('weather')) {
    const weather = new Weather(messageData.weather);
    globalData.weather = weather;
  }
}

socket.onclose = (event) => {
  console.log("Socket closed:");
  console.log(event);
}

socket.onerror = (error) => {
  console.log("Socket error:");
  console.log(error);
}

function frameCallback(): void {
  currentPanel.animationFrameTick();
  window.requestAnimationFrame(frameCallback);
}

function changeTabTo(tab: string) : void {
  currentPanel?.tearDown();

  if (tab === 'clock') {
    currentPanel = new ClockPanel(globalData);
    currentPanel.setUp();
  } else if (tab === 'weather') {
    currentPanel = new WeatherPanel(globalData);
    currentPanel.setUp();
  }
}

// Helper functions

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
