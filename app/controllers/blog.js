import Controller from '@ember/controller';
import { action } from '@ember/object';
import { inject as service } from '@ember/service';

import { dropTask } from 'ember-concurrency';
import { reads } from 'macro-decorators';

import { ajax_fail } from '../utils/ajax';

export default class BlogController extends Controller {
  @service store;

  @reads('dataTask.lastSuccessful.value') model;

  get hasData() {
    return this.dataTask.lastSuccessful && !this.dataTask.isRunning;
  }

  @action fetchData() {
    return this.dataTask.perform().catch(() => {
      // we ignore errors here because they are handled in the template already
    });
  }

  dataTask = dropTask(async () => {
    let issues = await ajax_fail('https://raw.githubusercontent.com/SeaQL/rustacean.info/refs/heads/main/issues.json');

    // console.log('issues', issues);

    return { issues };
  });
}
