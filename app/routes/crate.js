import { NotFoundError } from '@ember-data/adapter/error';
import Route from '@ember/routing/route';
import { inject as service } from '@ember/service';
import { ajax_fail, htmlDecode } from '../utils/ajax';

export default class CrateRoute extends Route {
  @service headData;
  @service router;
  @service store;

  async model(params, transition) {
    let crateName = params.crate_id;

    try {
      return (async () => {
        const data = await ajax_fail(
          `https://raw.githubusercontent.com/SeaQL/rustacean.info/main/related-articles/${crateName}.json`,
        );
        const res = await this.store.findRecord('crate', crateName);
        Object.assign(res, {
          articles: data.map(row => {
            row.title = htmlDecode(row.title);
            return row;
          }),
          link_contribute_articles: `https://github.com/SeaQL/rustacean.info/blob/main/CONTRIBUTING.md`,
          link_create_articles: `https://github.com/SeaQL/rustacean.info/new/main/related-articles`,
          link_edit_articles: `https://github.com/SeaQL/rustacean.info/edit/main/related-articles/${crateName}.json`,
        });
        return res;
      })();
    } catch (error) {
      if (error instanceof NotFoundError) {
        let title = `${crateName}: Crate not found`;
        this.router.replaceWith('catch-all', { transition, error, title });
      } else {
        let title = `${crateName}: Failed to load crate data`;
        this.router.replaceWith('catch-all', { transition, error, title, tryAgain: true });
      }
    }
  }

  setupController(controller, model) {
    super.setupController(...arguments);
    this.headData.crate = model;
  }

  resetController() {
    super.resetController(...arguments);
    this.headData.crate = null;
  }
}
