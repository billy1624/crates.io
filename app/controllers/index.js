import Controller from '@ember/controller';
import { action } from '@ember/object';
import { inject as service } from '@ember/service';

import { dropTask } from 'ember-concurrency';
import { reads } from 'macro-decorators';

import { ajax_fail } from '../utils/ajax';

export default class IndexController extends Controller {
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
    let data = await ajax_fail('https://raw.githubusercontent.com/SeaQL/rustacean.info/main/related-articles.json');

    shuffle(data);

    return data.slice(0, 100);
  });
}

function shuffle(array) {
  let currentIndex = array.length;

  // While there remain elements to shuffle...
  while (currentIndex != 0) {
    // Pick a remaining element...
    let randomIndex = Math.floor(Math.random() * currentIndex);
    currentIndex--;

    // And swap it with the current element.
    [array[currentIndex], array[randomIndex]] = [array[randomIndex], array[currentIndex]];
  }
}

function addCrates(store, crates) {
  for (let i = 0; i < crates.length; i++) {
    crates[i] = store.push(store.normalize('crate', crates[i]));
  }
}
