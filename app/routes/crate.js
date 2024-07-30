import { NotFoundError } from '@ember-data/adapter/error';
import Route from '@ember/routing/route';
import { inject as service } from '@ember/service';
import { ajax_fail } from '../utils/ajax';

export default class CrateRoute extends Route {
  @service headData;
  @service router;
  @service store;

  async model(params, transition) {
    let crateName = params.crate_id;

    try {
      return (async () => {
        const data = await ajax_fail(
          `https://raw.githubusercontent.com/billy1624/crates.io/rustacean.info/public/related-articles/${crateName}.json`,
        );
        const res = await this.store.findRecord('crate', crateName);
        Object.assign(res, {
          articles: data,
          link_contribute_articles: `https://github.com/billy1624/crates.io/blob/rustacean.info/public/related-articles/_CONTRIBUTING.md`,
          link_create_articles: `https://github.com/billy1624/crates.io/tree/rustacean.info/public/related-articles`,
          link_edit_articles: `https://github.com/billy1624/crates.io/edit/rustacean.info/public/related-articles/${crateName}.json`,
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
