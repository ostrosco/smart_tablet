import * as dayjs from 'dayjs';


function updateTime() {
  // set date/time in DOM element
  document.getElementById("date").innerHTML = dayjs().format('h:mm:ss A');
}

let updateTimeCallback = () => {
    updateTime();
    window.requestAnimationFrame(updateTimeCallback);
}

window.requestAnimationFrame(updateTimeCallback);