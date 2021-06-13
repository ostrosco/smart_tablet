import * as dayjs from 'dayjs';
import * as advancedFormat from 'dayjs/plugin/advancedFormat';
import './style.css';

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
