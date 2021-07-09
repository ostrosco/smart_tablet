import * as dayjs from 'dayjs';
import * as advancedFormat from 'dayjs/plugin/advancedFormat';
import { ContentPanel } from './contentPanel';
import { GlobalData } from './globalData';

dayjs.extend(advancedFormat);

const clockPanelTemplate = 
`<div id="clock" class="clock-blue">
  <div id="clock-time"></div>
  <div id="clock-sub-row-1">
    <div id="clock-day"></div>
    <div id="clock-location" class="flex-align-right"></div>
  </div>
  <div id="clock-sub-row-2">
    <div id="clock-date"></div>
    <div id="clock-weather" class="flex-align-right"></div>
    <div id="clock-temperature"></div>
  </div>
</div>`;

export class ClockPanel extends ContentPanel {

  constructor (private globalData: GlobalData) {
    super();
  }

  private _isSetUp: boolean = false;

  public get isSetUp(): boolean {
    return this._isSetUp;
  }

  public setUp(): void {
    document.getElementById('content').innerHTML = clockPanelTemplate;
    this._isSetUp = true;
  }

  public tearDown(): void {
    // no tear down
  }

  public animationFrameTick(): void {
    if (!this.isSetUp) {
      return;
    }

    var currentTime = dayjs();
    document.getElementById("clock-time").innerHTML = currentTime.format('h:mm:ss A');
    document.getElementById("clock-day").innerHTML = currentTime.format('dddd');
    document.getElementById("clock-date").innerHTML = currentTime.format('MMMM Do');

    if(this.globalData.location) {
      const lat = this.globalData.location.coords.latitude;
      const long = this.globalData.location.coords.longitude;
      const NSStr = lat >= 0 ? 'N' : 'S';
      const EWStr = long >= 0 ? 'E' : 'W';
      document.getElementById("clock-location").innerHTML = `(${Math.abs(long).toFixed(2)}&#176;${EWStr}, ${Math.abs(lat).toFixed(2)}&#176;${NSStr})`;
    }

    if(this.globalData.weather) {
      document.getElementById("clock-temperature").innerHTML = `${Math.round(this.globalData.weather.main.temp)}&#176;`
      document.getElementById("clock-weather").innerHTML = this.globalData.weather.weather[0].description;      
    }
  }
}