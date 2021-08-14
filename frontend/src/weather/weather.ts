import * as dayjs from 'dayjs';
import * as advancedFormat from 'dayjs/plugin/advancedFormat';
import { ContentPanel } from '../contentPanel';
import { GlobalData } from '../globalData';
import weatherTemplateHtml from './weather.template.html';

dayjs.extend(advancedFormat);

export class WeatherPanel extends ContentPanel {

  constructor (private globalData: GlobalData) {
    super();
  }

  private _isSetUp = false;

  public get isSetUp(): boolean {
    return this._isSetUp;
  }

  public setUp(): void {
    document.getElementById('content').innerHTML = weatherTemplateHtml;

    let weatherElement = document.getElementById('weather');

    for (let forecast of this.globalData.weather?.forecast) { // TODO: null check
      let forecastRow = document.createElement("p");
      forecastRow.innerHTML = `${forecast.date}: High ${forecast.max_temp}&#176; Low ${forecast.min_temp}&#176; ${forecast.description}`;

      weatherElement.appendChild(forecastRow);
    }

    this._isSetUp = true;
  }

  public tearDown(): void {
    this._isSetUp = false;

    // no tear down
  }

  public animationFrameTick(): void {
    if (!this.isSetUp) {
      return;
    }
  }
}