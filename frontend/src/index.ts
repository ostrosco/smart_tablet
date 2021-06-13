import * as dayjs from 'dayjs';
import * as advancedFormat from 'dayjs/plugin/advancedFormat';
import './style.css';

dayjs.extend(advancedFormat);

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
